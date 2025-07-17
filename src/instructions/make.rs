use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvars::rent::Rent,
    ProgramResult,
};
use pinocchio_system::instructions::CreateAccount;
use pinocchio_token::{instructions::Transfer as TokenTransfer, state::TokenAccount, ID};

use crate::{
    error::EscrowErrorCode,
    states::{DataLen, Escrow, EscrowType},
};

pub fn make_escrow(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    use pinocchio::sysvars::{clock::Clock, Sysvar};
    let [maker_account, maker_token_a_ata, escrow_account, escrow_token_a_ata, token_a_mint, token_b_mint, _system_program, _rent_sysvar, _remaing @ ..] =
        &accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Validation for accounts
    if !maker_account.is_signer() {
        return Err(EscrowErrorCode::InvalidMaker.into());
    }

    if !escrow_account.data_is_empty() {
        return Err(EscrowErrorCode::EscrowAlreadyExists.into());
    }

    if unsafe { token_a_mint.owner() } != &ID || unsafe { token_b_mint.owner() } != &ID {
        return Err(EscrowErrorCode::InvalidTokenOwner.into());
    }
    if maker_token_a_ata.data_is_empty() {
        return Err(EscrowErrorCode::InvalidMakerTokenAccount.into());
    }

    let maker_token_a_account: &TokenAccount =
        unsafe { TokenAccount::from_account_info_unchecked(maker_token_a_ata) }?;
    if maker_token_a_account.owner() != maker_account.key() {
        return Err(EscrowErrorCode::InvalidTokenOwner.into());
    }

    let ix_data = MakeEscrowIx::unpack(_instruction_data)?;

    Escrow::validate_escrow_pda(
        escrow_account.key(),
        maker_account.key(),
        &ix_data.bump,
        &ix_data.seed,
    )?;

    let bump_array = [ix_data.bump];
    let seed = [
        Seed::from(Escrow::PREFIX.as_bytes()),
        Seed::from(maker_account.key()),
        Seed::from(&ix_data.seed),
        Seed::from(&bump_array),
    ];
    let signer = Signer::from(&seed);

    // Create the PDA account
    CreateAccount {
        from: maker_account,
        to: escrow_account,
        lamports: Rent::get()?.minimum_balance(Escrow::LEN),
        space: Escrow::LEN as u64,
        owner: &crate::ID,
    }
    .invoke_signed(&[signer])?;

    // Set start_time and end_time for Dutch auction
    let (start_time, end_time) = if ix_data.escrow_type == EscrowType::DutchAuction {
        let now = Clock::get()?.unix_timestamp as u64;
        (now, now + ix_data.duration)
    } else {
        (0, 0)
    };

    Escrow::initialize(
        escrow_account,
        &ix_data,
        ix_data.seed,
        *token_a_mint.key(),
        *token_b_mint.key(),
        *maker_account.key(),
        start_time,
        end_time,
    )?;

    TokenTransfer {
        from: maker_token_a_ata,
        to: escrow_token_a_ata,
        authority: maker_account,
        amount: ix_data.token_a_amount,
    }
    .invoke()?;

    Ok(())
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MakeEscrowIx {
    pub escrow_type: EscrowType,
    pub token_a_amount: u64,
    pub token_b_amount: u64,
    pub seed: [u8; 2],
    pub bump: u8,
    // Dutch auction specific fields
    pub end_price: u64, // Minimum amount of token B required
    pub duration: u64,  // Auction duration in seconds (user input)
}

impl MakeEscrowIx {
    pub const LEN: usize = 1 + 8 + 8 + 2 + 1 + 8 + 8 + 8; // Added 24 bytes for Dutch auction fields

    pub fn new(
        escrow_type: EscrowType,
        token_a_amount: u64,
        token_b_amount: u64,
        bump: u8,
        seed: [u8; 2],
    ) -> Self {
        Self {
            escrow_type,
            token_a_amount,
            token_b_amount,
            seed,
            bump,
            end_price: 0,
            duration: 0,
        }
    }

    pub fn new_dutch_auction(
        token_a_amount: u64,
        start_price: u64,
        end_price: u64,
        start_time: u64,
        end_time: u64,
        bump: u8,
        seed: [u8; 2],
    ) -> Self {
        Self {
            escrow_type: EscrowType::DutchAuction,
            token_a_amount,
            token_b_amount: start_price, // Use start_price as token_b_amount
            seed,
            bump,
            end_price,
            duration: end_time - start_time,
        }
    }

    pub fn pack(&self) -> [u8; Self::LEN] {
        let mut data = [0u8; Self::LEN];
        data[0] = self.escrow_type as u8;
        data[1..9].copy_from_slice(&self.token_a_amount.to_le_bytes());
        data[9..17].copy_from_slice(&self.token_b_amount.to_le_bytes());
        data[17..19].copy_from_slice(&self.seed);
        data[19] = self.bump;

        // Pack Dutch auction fields
        let end_price_bytes = self.end_price.to_le_bytes();
        data[20..28].copy_from_slice(&end_price_bytes);
        let duration_bytes = self.duration.to_le_bytes();
        data[28..36].copy_from_slice(&duration_bytes);

        data
    }

    pub fn unpack(data: &[u8]) -> Result<Self, ProgramError> {
        let escrow_type =
            EscrowType::try_from(data[0]).map_err(|_| ProgramError::InvalidInstructionData)?;
        let token_a_amount = u64::from_le_bytes(
            data[1..9]
                .try_into()
                .map_err(|_| ProgramError::InvalidInstructionData)?,
        );
        let token_b_amount = u64::from_le_bytes(
            data[9..17]
                .try_into()
                .map_err(|_| ProgramError::InvalidInstructionData)?,
        );
        let seed = data[17..19]
            .try_into()
            .map_err(|_| ProgramError::InvalidInstructionData)?;
        let bump = data[19];

        // Unpack Dutch auction fields
        let end_price = u64::from_le_bytes(
            data[20..28]
                .try_into()
                .map_err(|_| ProgramError::InvalidInstructionData)?,
        );
        let duration = u64::from_le_bytes(
            data[28..36]
                .try_into()
                .map_err(|_| ProgramError::InvalidInstructionData)?,
        );

        Ok(Self {
            escrow_type,
            token_a_amount,
            token_b_amount,
            seed,
            bump,
            end_price,
            duration,
        })
    }
}
