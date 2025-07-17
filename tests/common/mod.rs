use anyhow::Result;
use escrow_suite::{instructions::MakeEscrowIx, states::EscrowType, ID};
use litesvm::LiteSVM;
use litesvm_token::{spl_token, CreateAssociatedTokenAccount, CreateMint, MintTo};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    message::{v0, VersionedMessage},
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    system_program,
    sysvar::clock::Clock,
    transaction::VersionedTransaction,
};
use spl_associated_token_account::get_associated_token_address;

pub fn setup_svm_and_program() -> (LiteSVM, Keypair, Pubkey) {
    let mut svm = LiteSVM::new();
    let fee_payer = Keypair::new();
    svm.airdrop(&fee_payer.pubkey(), 100000000).unwrap();

    let program_id = Pubkey::from(ID);
    svm.add_program_from_file(program_id, "./target/deploy/escrow_suite.so")
        .unwrap();

    (svm, fee_payer, program_id)
}

pub fn setup_mint(svm: &mut LiteSVM, payer: &Keypair) -> anyhow::Result<Pubkey> {
    let mint = CreateMint::new(svm, payer)
        .decimals(9)
        .token_program_id(&spl_token::ID)
        .send()
        .map_err(|e| anyhow::anyhow!("Failed to create mint {:?}", e))?;
    Ok(mint)
}

pub fn setup_ata(
    svm: &mut LiteSVM,
    mint: &Pubkey,
    user: &Pubkey,
    payer: &Keypair,
) -> Result<Pubkey, anyhow::Error> {
    CreateAssociatedTokenAccount::new(svm, payer, mint)
        .owner(user)
        .send()
        .map_err(|_| anyhow::anyhow!("Failed to create associated token account"))
}

pub fn mint_to(
    svm: &mut LiteSVM,
    mint: &Pubkey,
    authority: &Keypair,
    to: &Pubkey,
    amount: u64,
) -> Result<(), anyhow::Error> {
    MintTo::new(svm, authority, mint, to, amount)
        .send()
        .map_err(|e| anyhow::anyhow!("Failed to mint {:?}", e))?;
    Ok(())
}

pub fn display_user_balance_and_ata_balance(
    svm: &LiteSVM,
    user: &Pubkey,
    mint1: &Pubkey,
    mint2: &Pubkey,
) -> Result<(), anyhow::Error> {
    let sol_balance = svm.get_balance(user);
    println!("User: {:?}", user);
    println!("User balance: {:?}", sol_balance);

    let ata1 = get_associated_token_address(user, mint1);
    let ata2 = get_associated_token_address(user, mint2);

    if let Some(ata1_account) = svm.get_account(&ata1) {
        if ata1_account.data.len() >= 72 {
            println!(
                "ATA1 balance: {:?}",
                u64::from_le_bytes(ata1_account.data[64..72].try_into().unwrap())
            );
        } else {
            println!("ATA1 not found");
        };
    } else {
        println!("ATA1 not found");
    }

    if let Some(ata2_account) = svm.get_account(&ata2) {
        if ata2_account.data.len() >= 72 {
            println!(
                "ATA2 balance: {:?}",
                u64::from_le_bytes(ata2_account.data[64..72].try_into().unwrap())
            );
        } else {
            println!("ATA2 not found");
        };
    } else {
        println!("ATA2 not found");
    }

    Ok(())
}

pub struct EscrowTestSetup {
    pub svm: LiteSVM,
    pub maker: Keypair,
    pub taker: Keypair,
    pub program_id: Pubkey,
    pub token_a_mint: Pubkey,
    pub token_b_mint: Pubkey,
    pub maker_token_a_ata: Pubkey,
    pub maker_token_b_ata: Pubkey,
    pub taker_token_a_ata: Pubkey,
    pub taker_token_b_ata: Pubkey,
    pub escrow_pda: Pubkey,
    pub escrow_token_a_ata: Pubkey,
    pub bump: u8,
    pub seed: [u8; 2],
}

impl EscrowTestSetup {
    pub fn new() -> Result<Self> {
        let (mut svm, maker, program_id) = setup_svm_and_program();

        let token_a_mint = setup_mint(&mut svm, &maker)
            .map_err(|e| anyhow::anyhow!("Failed to setup mint: {:?}", e))?;
        let token_b_mint = setup_mint(&mut svm, &maker)
            .map_err(|e| anyhow::anyhow!("Failed to setup mint: {:?}", e))?;

        let maker_token_a_ata = setup_ata(&mut svm, &token_a_mint, &maker.pubkey(), &maker)
            .map_err(|e| anyhow::anyhow!("Failed to setup ATA: {:?}", e))?;
        let maker_token_b_ata = setup_ata(&mut svm, &token_b_mint, &maker.pubkey(), &maker)
            .map_err(|e| anyhow::anyhow!("Failed to setup ATA: {:?}", e))?;

        // Mint initial tokens to maker
        mint_to(&mut svm, &token_a_mint, &maker, &maker_token_a_ata, 10000)
            .map_err(|e| anyhow::anyhow!("Failed to mint tokens: {:?}", e))?;
        mint_to(&mut svm, &token_b_mint, &maker, &maker_token_b_ata, 10000)
            .map_err(|e| anyhow::anyhow!("Failed to mint tokens: {:?}", e))?;

        let seed: [u8; 2] = [0, 0];
        let (escrow_pda, bump) =
            Pubkey::find_program_address(&[b"Escrow", maker.pubkey().as_ref(), &seed], &program_id);

        let escrow_token_a_ata = setup_ata(&mut svm, &token_a_mint, &escrow_pda, &maker)
            .map_err(|e| anyhow::anyhow!("Failed to setup escrow ATA: {:?}", e))?;

        // Setup taker
        let taker = Keypair::new();
        svm.airdrop(&taker.pubkey(), 10000000)
            .map_err(|e| anyhow::anyhow!("Failed to airdrop: {:?}", e))?;

        let taker_token_a_ata = setup_ata(&mut svm, &token_a_mint, &taker.pubkey(), &taker)
            .map_err(|e| anyhow::anyhow!("Failed to setup taker ATA: {:?}", e))?;
        let taker_token_b_ata = setup_ata(&mut svm, &token_b_mint, &taker.pubkey(), &taker)
            .map_err(|e| anyhow::anyhow!("Failed to setup taker ATA: {:?}", e))?;

        // Mint tokens to taker
        mint_to(&mut svm, &token_a_mint, &maker, &taker_token_a_ata, 10000)
            .map_err(|e| anyhow::anyhow!("Failed to mint tokens to taker: {:?}", e))?;
        mint_to(&mut svm, &token_b_mint, &maker, &taker_token_b_ata, 10000)
            .map_err(|e| anyhow::anyhow!("Failed to mint tokens to taker: {:?}", e))?;

        Ok(Self {
            svm,
            maker,
            taker,
            program_id,
            token_a_mint,
            token_b_mint,
            maker_token_a_ata,
            maker_token_b_ata,
            taker_token_a_ata,
            taker_token_b_ata,
            escrow_pda,
            escrow_token_a_ata,
            bump,
            seed,
        })
    }

    pub fn create_escrow(
        &mut self,
        escrow_type: EscrowType,
        token_a_amount: u64,
        token_b_amount: u64,
    ) -> Result<()> {
        let mut ix_data = [0u8; MakeEscrowIx::LEN + 1];
        ix_data[0] = 0x01;

        let ix = MakeEscrowIx::new(
            escrow_type,
            token_a_amount,
            token_b_amount,
            self.bump,
            self.seed,
        );

        ix_data[1..].copy_from_slice(&ix.pack());

        let accounts = vec![
            AccountMeta::new(self.maker.pubkey(), true),
            AccountMeta::new(self.maker_token_a_ata, false),
            AccountMeta::new(self.escrow_pda, false),
            AccountMeta::new(self.escrow_token_a_ata, false),
            AccountMeta::new_readonly(self.token_a_mint, false),
            AccountMeta::new_readonly(self.token_b_mint, false),
            AccountMeta::new(self.program_id, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(spl_token::ID, false),
        ];

        let instruction = Instruction {
            program_id: self.program_id,
            accounts,
            data: ix_data.to_vec(),
        };

        let msg = v0::Message::try_compile(
            &self.maker.pubkey(),
            &[instruction],
            &[],
            self.svm.latest_blockhash(),
        )
        .map_err(|e| anyhow::anyhow!("Failed to compile message: {:?}", e))?;

        let tx = VersionedTransaction::try_new(
            VersionedMessage::V0(msg),
            &[self.maker.insecure_clone()],
        )
        .map_err(|e| anyhow::anyhow!("Failed to create transaction: {:?}", e))?;

        self.svm
            .send_transaction(tx)
            .map_err(|e| anyhow::anyhow!("Failed to send transaction: {:?}", e))?;
        Ok(())
    }

    pub fn create_dutch_auction_escrow(
        &mut self,
        token_a_amount: u64,
        start_price: u64,
        end_price: u64,
        duration: u64,
    ) -> Result<()> {
        let mut ix_data = [0u8; MakeEscrowIx::LEN + 1];
        ix_data[0] = 0x01;

        let ix = MakeEscrowIx {
            escrow_type: EscrowType::DutchAuction,
            token_a_amount,
            token_b_amount: start_price, // Use start_price as token_b_amount
            seed: self.seed,
            bump: self.bump,
            end_price,
            duration,
        };

        ix_data[1..].copy_from_slice(&ix.pack());

        let accounts = vec![
            AccountMeta::new(self.maker.pubkey(), true),
            AccountMeta::new(self.maker_token_a_ata, false),
            AccountMeta::new(self.escrow_pda, false),
            AccountMeta::new(self.escrow_token_a_ata, false),
            AccountMeta::new_readonly(self.token_a_mint, false),
            AccountMeta::new_readonly(self.token_b_mint, false),
            AccountMeta::new(self.program_id, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(spl_token::ID, false),
        ];

        let instruction = Instruction {
            program_id: self.program_id,
            accounts,
            data: ix_data.to_vec(),
        };

        let msg = v0::Message::try_compile(
            &self.maker.pubkey(),
            &[instruction],
            &[],
            self.svm.latest_blockhash(),
        )
        .map_err(|e| anyhow::anyhow!("Failed to compile message: {:?}", e))?;

        let tx = VersionedTransaction::try_new(
            VersionedMessage::V0(msg),
            &[self.maker.insecure_clone()],
        )
        .map_err(|e| anyhow::anyhow!("Failed to create transaction: {:?}", e))?;

        self.svm
            .send_transaction(tx)
            .map_err(|e| anyhow::anyhow!("Failed to send transaction: {:?}", e))?;
        Ok(())
    }

    pub fn take_escrow(&mut self) -> Result<()> {
        self.take_escrow_with_amounts(0, 0)
    }

    pub fn take_escrow_with_amounts(
        &mut self,
        token_a_amount: u64,
        token_b_amount: u64,
    ) -> Result<()> {
        let accounts = vec![
            AccountMeta::new(self.escrow_pda, false),
            AccountMeta::new(self.escrow_token_a_ata, false),
            AccountMeta::new(self.maker.pubkey(), false),
            AccountMeta::new(self.maker_token_b_ata, false),
            AccountMeta::new(self.taker.pubkey(), true),
            AccountMeta::new(self.taker_token_a_ata, false),
            AccountMeta::new(self.taker_token_b_ata, false),
            AccountMeta::new(self.program_id, false),
            AccountMeta::new(self.program_id, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(spl_token::ID, false),
        ];

        // Create instruction data for take escrow
        let mut ix_data = vec![0x02]; // Discriminator for take instruction

        // Add instruction data for Dutch auction
        if token_a_amount > 0 || token_b_amount > 0 {
            use escrow_suite::instructions::TakeEscrowIx;
            let take_ix = TakeEscrowIx::new(
                escrow_suite::states::EscrowType::DutchAuction,
                token_a_amount,
                token_b_amount,
            );
            ix_data.extend_from_slice(&take_ix.pack());
        }

        let instruction = Instruction {
            program_id: self.program_id,
            accounts,
            data: ix_data,
        };

        let msg = v0::Message::try_compile(
            &self.taker.pubkey(),
            &[instruction],
            &[],
            self.svm.latest_blockhash(),
        )
        .map_err(|e| anyhow::anyhow!("Failed to compile message: {:?}", e))?;

        let tx = VersionedTransaction::try_new(
            VersionedMessage::V0(msg),
            &[self.taker.insecure_clone()],
        )
        .map_err(|e| anyhow::anyhow!("Failed to create transaction: {:?}", e))?;

        self.svm
            .send_transaction(tx)
            .map_err(|e| anyhow::anyhow!("Failed to send transaction: {:?}", e))?;
        Ok(())
    }

    /// Take a partial amount from a partial escrow
    pub fn take_partial_escrow(&mut self, token_a_amount: u64) -> Result<()> {
        let accounts = vec![
            AccountMeta::new(self.escrow_pda, false),
            AccountMeta::new(self.escrow_token_a_ata, false),
            AccountMeta::new(self.maker.pubkey(), false),
            AccountMeta::new(self.maker_token_b_ata, false),
            AccountMeta::new(self.taker.pubkey(), true),
            AccountMeta::new(self.taker_token_a_ata, false),
            AccountMeta::new(self.taker_token_b_ata, false),
            AccountMeta::new(self.program_id, false),
            AccountMeta::new(self.program_id, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(spl_token::ID, false),
        ];

        // Create instruction data for partial take
        let mut ix_data = vec![0x02]; // Discriminator for take instruction

        use escrow_suite::instructions::TakeEscrowIx;
        let take_ix = TakeEscrowIx::new(
            escrow_suite::states::EscrowType::Partial,
            token_a_amount,
            0, // token_b_amount will be calculated by the program
        );
        ix_data.extend_from_slice(&take_ix.pack());

        let instruction = Instruction {
            program_id: self.program_id,
            accounts,
            data: ix_data,
        };

        let msg = v0::Message::try_compile(
            &self.taker.pubkey(),
            &[instruction],
            &[],
            self.svm.latest_blockhash(),
        )
        .map_err(|e| anyhow::anyhow!("Failed to compile message: {:?}", e))?;

        let tx = VersionedTransaction::try_new(
            VersionedMessage::V0(msg),
            &[self.taker.insecure_clone()],
        )
        .map_err(|e| anyhow::anyhow!("Failed to create transaction: {:?}", e))?;

        self.svm
            .send_transaction(tx)
            .map_err(|e| anyhow::anyhow!("Failed to send transaction: {:?}", e))?;
        Ok(())
    }

    pub fn display_balances(&self) -> Result<()> {
        println!("=== Maker Balances ===");
        display_user_balance_and_ata_balance(
            &self.svm,
            &self.maker.pubkey(),
            &self.token_a_mint,
            &self.token_b_mint,
        )?;

        println!("=== Taker Balances ===");
        display_user_balance_and_ata_balance(
            &self.svm,
            &self.taker.pubkey(),
            &self.token_a_mint,
            &self.token_b_mint,
        )?;

        println!("=== Escrow PDA Balances ===");
        display_user_balance_and_ata_balance(
            &self.svm,
            &self.escrow_pda,
            &self.token_a_mint,
            &self.token_b_mint,
        )?;

        Ok(())
    }

    pub fn get_balance(&self, user: &Pubkey, mint: &Pubkey) -> u64 {
        let ata = get_associated_token_address(user, mint);
        if let Some(account) = self.svm.get_account(&ata) {
            if account.data.len() >= 72 {
                u64::from_le_bytes(account.data[64..72].try_into().unwrap())
            } else {
                0
            }
        } else {
            0
        }
    }

    pub fn get_maker_token_a_balance(&self) -> u64 {
        self.get_balance(&self.maker.pubkey(), &self.token_a_mint)
    }

    pub fn get_maker_token_b_balance(&self) -> u64 {
        self.get_balance(&self.maker.pubkey(), &self.token_b_mint)
    }

    pub fn get_taker_token_a_balance(&self) -> u64 {
        self.get_balance(&self.taker.pubkey(), &self.token_a_mint)
    }

    pub fn get_taker_token_b_balance(&self) -> u64 {
        self.get_balance(&self.taker.pubkey(), &self.token_b_mint)
    }

    pub fn get_escrow_token_a_balance(&self) -> u64 {
        self.get_balance(&self.escrow_pda, &self.token_a_mint)
    }

    pub fn get_escrow_token_b_balance(&self) -> u64 {
        self.get_balance(&self.escrow_pda, &self.token_b_mint)
    }

    pub fn verify_simple_escrow_balances(
        &self,
        token_a_amount: u64,
        token_b_amount: u64,
        stage: &str,
    ) -> Result<()> {
        let maker_token_a = self.get_maker_token_a_balance();
        let maker_token_b = self.get_maker_token_b_balance();
        let taker_token_a = self.get_taker_token_a_balance();
        let taker_token_b = self.get_taker_token_b_balance();
        let escrow_token_a = self.get_escrow_token_a_balance();
        let escrow_token_b = self.get_escrow_token_b_balance();

        println!("=== Balance Verification for {} ===", stage);
        println!("Maker Token A: {}", maker_token_a);
        println!("Maker Token B: {}", maker_token_b);
        println!("Taker Token A: {}", taker_token_a);
        println!("Taker Token B: {}", taker_token_b);
        println!("Escrow Token A: {}", escrow_token_a);
        println!("Escrow Token B: {}", escrow_token_b);

        match stage {
            "initial" => {
                // Initial state: maker has 10000 of each token, taker has 10000 of each token
                assert_eq!(
                    maker_token_a, 10000,
                    "Maker should have 10000 Token A initially"
                );
                assert_eq!(
                    maker_token_b, 10000,
                    "Maker should have 10000 Token B initially"
                );
                assert_eq!(
                    taker_token_a, 10000,
                    "Taker should have 10000 Token A initially"
                );
                assert_eq!(
                    taker_token_b, 10000,
                    "Taker should have 10000 Token B initially"
                );
                assert_eq!(escrow_token_a, 0, "Escrow should have 0 Token A initially");
                assert_eq!(escrow_token_b, 0, "Escrow should have 0 Token B initially");
            }
            "after_creation" => {
                // After creation: maker's token A should be reduced, escrow should have token A
                assert_eq!(
                    maker_token_a,
                    10000 - token_a_amount,
                    "Maker Token A should be reduced by escrow amount"
                );
                assert_eq!(
                    maker_token_b, 10000,
                    "Maker Token B should remain unchanged"
                );
                assert_eq!(
                    taker_token_a, 10000,
                    "Taker Token A should remain unchanged"
                );
                assert_eq!(
                    taker_token_b, 10000,
                    "Taker Token B should remain unchanged"
                );
                assert_eq!(
                    escrow_token_a, token_a_amount,
                    "Escrow should have the escrow amount of Token A"
                );
                assert_eq!(escrow_token_b, 0, "Escrow should have 0 Token B");
            }
            "after_take" => {
                // After take: taker should have token A, maker should have token B, escrow should be empty
                assert_eq!(
                    maker_token_a,
                    10000 - token_a_amount,
                    "Maker Token A should remain reduced"
                );
                assert_eq!(
                    maker_token_b,
                    10000 + token_b_amount,
                    "Maker Token B should be increased by payment amount"
                );
                assert_eq!(
                    taker_token_a,
                    10000 + token_a_amount,
                    "Taker Token A should be increased by escrow amount"
                );
                assert_eq!(
                    taker_token_b,
                    10000 - token_b_amount,
                    "Taker Token B should be reduced by payment amount"
                );
                assert_eq!(escrow_token_a, 0, "Escrow should have 0 Token A after take");
                assert_eq!(escrow_token_b, 0, "Escrow should have 0 Token B after take");
            }
            _ => return Err(anyhow::anyhow!("Unknown stage: {}", stage)),
        }

        println!("✅ Balance verification passed for {}", stage);
        Ok(())
    }

    pub fn verify_dutch_auction_balances(
        &self,
        token_a_amount: u64,
        expected_payment: u64,
        stage: &str,
    ) -> Result<()> {
        let maker_token_a = self.get_maker_token_a_balance();
        let maker_token_b = self.get_maker_token_b_balance();
        let taker_token_a = self.get_taker_token_a_balance();
        let taker_token_b = self.get_taker_token_b_balance();
        let escrow_token_a = self.get_escrow_token_a_balance();
        let escrow_token_b = self.get_escrow_token_b_balance();

        println!("=== Dutch Auction Balance Verification for {} ===", stage);
        println!("Maker Token A: {}", maker_token_a);
        println!("Maker Token B: {}", maker_token_b);
        println!("Taker Token A: {}", taker_token_a);
        println!("Taker Token B: {}", taker_token_b);
        println!("Escrow Token A: {}", escrow_token_a);
        println!("Escrow Token B: {}", escrow_token_b);

        match stage {
            "initial" => {
                // Initial state: maker has 10000 of each token, taker has 10000 of each token
                assert_eq!(
                    maker_token_a, 10000,
                    "Maker should have 10000 Token A initially"
                );
                assert_eq!(
                    maker_token_b, 10000,
                    "Maker should have 10000 Token B initially"
                );
                assert_eq!(
                    taker_token_a, 10000,
                    "Taker should have 10000 Token A initially"
                );
                assert_eq!(
                    taker_token_b, 10000,
                    "Taker should have 10000 Token B initially"
                );
                assert_eq!(escrow_token_a, 0, "Escrow should have 0 Token A initially");
                assert_eq!(escrow_token_b, 0, "Escrow should have 0 Token B initially");
            }
            "after_creation" => {
                // After creation: maker's token A should be reduced, escrow should have token A
                assert_eq!(
                    maker_token_a,
                    10000 - token_a_amount,
                    "Maker Token A should be reduced by escrow amount"
                );
                assert_eq!(
                    maker_token_b, 10000,
                    "Maker Token B should remain unchanged"
                );
                assert_eq!(
                    taker_token_a, 10000,
                    "Taker Token A should remain unchanged"
                );
                assert_eq!(
                    taker_token_b, 10000,
                    "Taker Token B should remain unchanged"
                );
                assert_eq!(
                    escrow_token_a, token_a_amount,
                    "Escrow should have the escrow amount of Token A"
                );
                assert_eq!(escrow_token_b, 0, "Escrow should have 0 Token B");
            }
            "after_take" => {
                // After take: taker should have token A, maker should have token B, escrow should be empty
                assert_eq!(
                    maker_token_a,
                    10000 - token_a_amount,
                    "Maker Token A should remain reduced"
                );
                assert_eq!(
                    maker_token_b,
                    10000 + expected_payment,
                    "Maker Token B should be increased by payment amount"
                );
                assert_eq!(
                    taker_token_a,
                    10000 + token_a_amount,
                    "Taker Token A should be increased by escrow amount"
                );
                assert_eq!(
                    taker_token_b,
                    10000 - expected_payment,
                    "Taker Token B should be reduced by payment amount"
                );
                assert_eq!(escrow_token_a, 0, "Escrow should have 0 Token A after take");
                assert_eq!(escrow_token_b, 0, "Escrow should have 0 Token B after take");
            }
            _ => return Err(anyhow::anyhow!("Unknown stage: {}", stage)),
        }

        println!("✅ Dutch auction balance verification passed for {}", stage);
        Ok(())
    }

    /// Verify balances for partial escrow operations
    pub fn verify_partial_escrow_balances(
        &self,
        total_token_a: u64,
        total_token_b: u64,
        taken_token_a: u64,
        taken_token_b: u64,
        remaining_token_a: u64,
        stage: &str,
    ) -> Result<()> {
        let maker_token_a = self.get_maker_token_a_balance();
        let maker_token_b = self.get_maker_token_b_balance();
        let taker_token_a = self.get_taker_token_a_balance();
        let taker_token_b = self.get_taker_token_b_balance();
        let escrow_token_a = self.get_escrow_token_a_balance();
        let escrow_token_b = self.get_escrow_token_b_balance();

        println!("=== Partial Escrow Balance Verification for {} ===", stage);
        println!("Total Token A: {}", total_token_a);
        println!("Total Token B: {}", total_token_b);
        println!("Taken Token A: {}", taken_token_a);
        println!("Taken Token B: {}", taken_token_b);
        println!("Remaining Token A: {}", remaining_token_a);
        println!("Maker Token A: {}", maker_token_a);
        println!("Maker Token B: {}", maker_token_b);
        println!("Taker Token A: {}", taker_token_a);
        println!("Taker Token B: {}", taker_token_b);
        println!("Escrow Token A: {}", escrow_token_a);
        println!("Escrow Token B: {}", escrow_token_b);

        match stage {
            "after_partial_take"
            | "after_first_take"
            | "after_second_take"
            | "after_third_take"
            | "after_small_take"
            | "after_large_take"
            | "after_half_take"
            | "after_almost_all_take"
            | "after_precision_take" => {
                // After partial take: verify proportional transfers
                assert_eq!(
                    maker_token_a,
                    10000 - total_token_a,
                    "Maker Token A should remain reduced by total escrow amount"
                );
                assert_eq!(
                    maker_token_b,
                    10000 + taken_token_b,
                    "Maker Token B should be increased by taken token B amount"
                );
                assert_eq!(
                    taker_token_a,
                    10000 + taken_token_a,
                    "Taker Token A should be increased by taken token A amount"
                );
                assert_eq!(
                    taker_token_b,
                    10000 - taken_token_b,
                    "Taker Token B should be reduced by taken token B amount"
                );
                assert_eq!(
                    escrow_token_a, remaining_token_a,
                    "Escrow should have remaining token A amount"
                );
                assert_eq!(escrow_token_b, 0, "Escrow should have 0 Token B");
            }
            _ => {
                // For other stages, use the same logic as simple escrow
                self.verify_simple_escrow_balances(total_token_a, total_token_b, stage)?;
            }
        }

        println!(
            "✅ Partial escrow balance verification passed for {}",
            stage
        );
        Ok(())
    }

    pub fn run_complete_escrow_test(
        escrow_type: EscrowType,
        token_a_amount: u64,
        token_b_amount: u64,
        should_take: bool,
    ) -> Result<()> {
        let mut setup = Self::new()?;

        println!(
            "=== Testing {} Escrow ===",
            match escrow_type {
                EscrowType::Simple => "Simple",
                EscrowType::Partial => "Partial",
                EscrowType::Oracle => "Oracle",
                EscrowType::DutchAuction => "Dutch Auction",
            }
        );
        println!("Token A Amount: {}", token_a_amount);
        println!("Token B Amount: {}", token_b_amount);

        // Verify initial balances
        setup.verify_simple_escrow_balances(token_a_amount, token_b_amount, "initial")?;

        // Create the escrow
        setup.create_escrow(escrow_type, token_a_amount, token_b_amount)?;

        // Verify balances after creation
        setup.verify_simple_escrow_balances(token_a_amount, token_b_amount, "after_creation")?;

        if should_take {
            // Take the escrow
            setup.take_escrow()?;

            // Verify balances after take
            setup.verify_simple_escrow_balances(token_a_amount, token_b_amount, "after_take")?;
        } else {
            println!("Escrow created successfully (take step skipped)");
        }

        Ok(())
    }

    /// Set the current time in the SVM for testing time-dependent features
    pub fn set_time(&mut self, timestamp: i64) -> Result<()> {
        // Create a new clock with the desired timestamp
        let clock = Clock {
            slot: 0,
            epoch_start_timestamp: timestamp,
            epoch: 0,
            leader_schedule_epoch: 0,
            unix_timestamp: timestamp,
        };

        // Update the clock sysvar in the SVM
        self.svm.set_sysvar(&clock);
        Ok(())
    }

    /// Get the current time from the SVM
    pub fn get_current_time(&self) -> Result<i64> {
        let clock = self.svm.get_sysvar::<Clock>();
        Ok(clock.unix_timestamp)
    }

    /// Advance time by the specified number of seconds
    pub fn advance_time(&mut self, seconds: i64) -> Result<()> {
        let current_time = self.get_current_time()?;
        self.set_time(current_time + seconds)
    }

    /// Calculate the expected Dutch auction price at a given time
    pub fn calculate_expected_dutch_price(
        &self,
        start_price: u64,
        end_price: u64,
        start_time: u64,
        end_time: u64,
        current_time: u64,
    ) -> u64 {
        if current_time <= start_time {
            return start_price;
        }
        if current_time >= end_time {
            return end_price;
        }

        let time_elapsed = current_time - start_time;
        let total_duration = end_time - start_time;
        let price_drop = start_price - end_price;
        let price_reduction = (price_drop as u128 * time_elapsed as u128) / total_duration as u128;

        start_price - (price_reduction as u64)
    }
}
