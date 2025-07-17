use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvars::{clock::Clock, Sysvar},
    ProgramResult,
};
use pinocchio_token::{instructions::Transfer as TokenTransfer, state::TokenAccount};

use crate::{
    error::EscrowErrorCode,
    states::{try_from_account_info_mut, Escrow, EscrowType},
};

pub fn take_escrow(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // Escrow and maker related accounts
    let [escrow_account, escrow_token_a_ata, maker_account, maker_token_b_ata, taker_account, taker_token_a_ata, taker_token_b_ata, _remaing @ ..] =
        &accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let escrow = unsafe { try_from_account_info_mut::<Escrow>(escrow_account) }?;

    Escrow::validate_escrow_pda(
        escrow_account.key(),
        maker_account.key(),
        &escrow.bump,
        &escrow.seed,
    )?;

    if !taker_account.is_signer() {
        return Err(EscrowErrorCode::InvalidMaker.into());
    }

    let taker_token_a_account: &TokenAccount =
        unsafe { TokenAccount::from_account_info_unchecked(taker_token_a_ata) }?;
    let taker_token_b_account: &TokenAccount =
        unsafe { TokenAccount::from_account_info_unchecked(taker_token_b_ata) }?;

    if taker_token_a_account.mint() != &escrow.token_a_mint {
        return Err(EscrowErrorCode::InvalidTokenMint.into());
    }

    if taker_token_b_account.mint() != &escrow.token_b_mint {
        return Err(EscrowErrorCode::InvalidTokenMint.into());
    }

    let bump_array = [escrow.bump];
    let seed = [
        Seed::from(Escrow::PREFIX.as_bytes()),
        Seed::from(maker_account.key()),
        Seed::from(&escrow.seed),
        Seed::from(&bump_array),
    ];
    let signer = Signer::from(&seed);

    match escrow.escrow_type {
        EscrowType::Simple => {
            if escrow.token_a_amount > taker_token_a_account.amount()
                || escrow.token_b_amount > taker_token_b_account.amount()
            {
                return Err(EscrowErrorCode::InsufficientFunds.into());
            }

            TokenTransfer {
                from: escrow_token_a_ata,
                to: taker_token_a_ata,
                authority: escrow_account,
                amount: escrow.token_a_amount,
            }
            .invoke_signed(&[signer.clone()])?;

            TokenTransfer {
                from: taker_token_b_ata,
                to: maker_token_b_ata,
                authority: taker_account,
                amount: escrow.token_b_amount,
            }
            .invoke()?;
        }
        // Here even if the change is not enough, we still transfer the token to the maker and reduce the escrow amount
        EscrowType::Partial => {
            let ix = TakeEscrowIx::unpack(instruction_data)?;

            if ix.token_a_amount > escrow.token_a_amount {
                return Err(EscrowErrorCode::InsufficientFunds.into());
            }

            let percentage = (ix.token_a_amount as u64 * 10000) / escrow.token_a_amount;
            let token_b_amount = (escrow.token_b_amount as u64 * percentage) / 10000;

            if token_b_amount > taker_token_b_account.amount() {
                return Err(EscrowErrorCode::InsufficientFunds.into());
            }

            TokenTransfer {
                from: escrow_token_a_ata,
                to: taker_token_a_ata,
                authority: escrow_account,
                amount: ix.token_a_amount,
            }
            .invoke_signed(&[signer.clone()])?;

            TokenTransfer {
                from: taker_token_b_ata,
                to: maker_token_b_ata,
                authority: taker_account,
                amount: token_b_amount,
            }
            .invoke()?;

            escrow.token_a_amount -= ix.token_a_amount;
            escrow.token_b_amount -= token_b_amount;
        }
        // In dutch auction, declining price mechanisms where the required amount of token B decreases over time until someone takes the offer.
        EscrowType::DutchAuction => {
            let ix = TakeEscrowIx::unpack(instruction_data)?;

            if ix.token_a_amount > taker_token_a_account.amount() {
                return Err(EscrowErrorCode::InsufficientFunds.into());
            }

            // Calculate current Dutch auction price
            let current_time = Clock::get()?.unix_timestamp as u64;
            let required_token_b_amount = escrow.get_required_token_b_amount(current_time);

            if ix.token_b_amount < required_token_b_amount {
                return Err(EscrowErrorCode::InsufficientFunds.into());
            }

            // Transfer token A from escrow to taker
            TokenTransfer {
                from: escrow_token_a_ata,
                to: taker_token_a_ata,
                authority: escrow_account,
                amount: ix.token_a_amount,
            }
            .invoke_signed(&[signer.clone()])?;

            TokenTransfer {
                from: taker_token_b_ata,
                to: maker_token_b_ata,
                authority: taker_account,
                amount: required_token_b_amount,
            }
            .invoke()?;
        }
        _ => {
            return Err(EscrowErrorCode::InvalidEscrowType.into());
        }
    }

    Ok(())
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct TakeEscrowIx {
    pub escrow_type: EscrowType,
    pub token_a_amount: u64,
    pub token_b_amount: u64,
}

impl TakeEscrowIx {
    pub const LEN: usize = 1 + 8 + 8;

    pub fn new(escrow_type: EscrowType, token_a_amount: u64, token_b_amount: u64) -> Self {
        Self {
            escrow_type,
            token_a_amount,
            token_b_amount,
        }
    }

    pub fn pack(&self) -> [u8; Self::LEN] {
        let mut data = [0u8; Self::LEN];
        data[0] = self.escrow_type as u8;
        data[1..9].copy_from_slice(&self.token_a_amount.to_le_bytes());
        data[9..17].copy_from_slice(&self.token_b_amount.to_le_bytes());
        data
    }

    pub fn unpack(data: &[u8]) -> Result<Self, ProgramError> {
        if data.len() != Self::LEN {
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(Self {
            escrow_type: EscrowType::try_from(data[0])?,
            token_a_amount: u64::from_le_bytes(data[1..9].try_into().unwrap()),
            token_b_amount: u64::from_le_bytes(data[9..17].try_into().unwrap()),
        })
    }
}
