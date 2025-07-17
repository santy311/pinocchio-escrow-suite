use crate::error::EscrowErrorCode;
use crate::instructions::MakeEscrowIx;
use crate::states::{try_from_account_info_mut, DataLen};
use pinocchio::account_info::AccountInfo;
use pinocchio::{msg, ProgramResult};
use pinocchio::{program_error::ProgramError, pubkey, pubkey::Pubkey};
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EscrowType {
    Simple = 0,
    Partial = 1,
    DutchAuction = 2,
    Oracle = 3,
}

impl TryFrom<u8> for EscrowType {
    type Error = ProgramError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Simple,
            1 => Self::Partial,
            2 => Self::DutchAuction,
            3 => Self::Oracle,
            _ => return Err(ProgramError::InvalidInstructionData),
        })
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Escrow {
    pub maker_pubkey: [u8; 32],
    pub seed: [u8; 2],
    pub escrow_type: EscrowType,
    pub token_a_mint: [u8; 32],
    pub token_a_amount: u64,
    pub token_b_mint: [u8; 32],
    pub token_b_amount: u64,
    pub bump: u8,
    // Dutch auction specific fields
    pub start_price: u64, // Initial amount of token B required
    pub end_price: u64,   // Minimum amount of token B required
    pub start_time: u64,  // Auction start timestamp (set by program)
    pub duration: u64,    // Auction duration in seconds (user input)
    pub end_time: u64,    // Auction end timestamp (computed as start_time + duration)
}

impl DataLen for Escrow {
    const LEN: usize = core::mem::size_of::<Self>();
}

impl Escrow {
    pub const PREFIX: &'static str = "Escrow";

    pub fn validate_escrow_pda(
        pda: &Pubkey,
        owner: &Pubkey,
        bump: &u8,
        seed: &[u8; 2],
    ) -> Result<(), ProgramError> {
        let seed_with_bump = &[Self::PREFIX.as_bytes(), owner, seed, &[*bump]];
        let derived = pubkey::create_program_address(seed_with_bump, &crate::ID)?;
        msg!("Derived: {:?}", derived);
        if derived != *pda {
            return Err(EscrowErrorCode::PdaMismatch.into());
        }
        Ok(())
    }

    pub fn new(
        escrow_type: EscrowType,
        maker_pubkey: [u8; 32],
        seed: [u8; 2],
        token_a_mint: [u8; 32],
        token_a_amount: u64,
        token_b_mint: [u8; 32],
        token_b_amount: u64,
        bump: u8,
    ) -> Self {
        Self {
            maker_pubkey,
            seed,
            escrow_type,
            token_a_mint,
            token_a_amount,
            token_b_mint,
            token_b_amount,
            bump,
            start_price: 0,
            end_price: 0,
            start_time: 0,
            duration: 0,
            end_time: 0,
        }
    }

    pub fn initialize(
        escrow_acc: &AccountInfo,
        ix_data: &MakeEscrowIx,
        seed: [u8; 2],
        token_a_mint: [u8; 32],
        token_b_mint: [u8; 32],
        maker_pubkey: [u8; 32],
        start_time: u64,
        end_time: u64,
    ) -> ProgramResult {
        let escrow = unsafe { try_from_account_info_mut::<Escrow>(escrow_acc) }?;

        escrow.maker_pubkey = maker_pubkey;
        escrow.seed = seed;
        escrow.escrow_type = ix_data.escrow_type;
        escrow.token_a_mint = token_a_mint;
        escrow.token_a_amount = ix_data.token_a_amount;
        escrow.token_b_mint = token_b_mint;
        escrow.token_b_amount = ix_data.token_b_amount;
        escrow.bump = ix_data.bump;

        // Initialize Dutch auction fields if needed
        if ix_data.escrow_type == EscrowType::DutchAuction {
            escrow.start_price = ix_data.token_b_amount;
            escrow.end_price = ix_data.end_price;
            escrow.duration = ix_data.duration;
            escrow.start_time = start_time;
            escrow.end_time = end_time;
        }

        Ok(())
    }

    /// Calculate current price for Dutch auction
    /// Returns the amount of token B required at current time
    pub fn calculate_dutch_price(&self, current_time: u64) -> u64 {
        match self.escrow_type {
            EscrowType::DutchAuction => {
                // Handle edge cases
                if current_time <= self.start_time {
                    return self.start_price;
                }
                if current_time >= self.end_time {
                    return self.end_price;
                }

                // Calculate time progress as a fraction
                let time_elapsed = current_time - self.start_time;
                let total_duration = self.end_time - self.start_time;

                // Calculate price drop using safe arithmetic
                let price_drop = self.start_price - self.end_price;

                // Use multiplication before division to maintain precision
                // Formula: current_price = start_price - (price_drop * time_elapsed / total_duration)
                let price_reduction =
                    (price_drop as u128 * time_elapsed as u128) / total_duration as u128;

                // Convert back to u64 safely
                self.start_price - (price_reduction as u64)
            }
            _ => self.token_b_amount, // For non-Dutch auctions, return the fixed amount
        }
    }

    /// Simplified Dutch auction with linear price decay (more gas efficient)
    pub fn simple_dutch_price(&self, current_time: u64, decay_rate: u64, min_price: u64) -> u64 {
        match self.escrow_type {
            EscrowType::DutchAuction => {
                if current_time <= self.start_time {
                    return self.start_price;
                }

                let time_elapsed = current_time - self.start_time;
                let total_decay = decay_rate.saturating_mul(time_elapsed);

                // Ensure we don't go below minimum price
                self.start_price.saturating_sub(total_decay).max(min_price)
            }
            _ => self.token_b_amount,
        }
    }

    /// Get the current required amount of token B for this escrow
    pub fn get_required_token_b_amount(&self, current_time: u64) -> u64 {
        match self.escrow_type {
            EscrowType::DutchAuction => self.calculate_dutch_price(current_time),
            _ => self.token_b_amount,
        }
    }

    // pub fn pack(&self) -> [u8; Self::LEN] {
    //     let mut data = [0u8; Self::LEN];
    //     data[0..32].copy_from_slice(&self.maker);
    //     data[32..34].copy_from_slice(&self.seed);
    //     data[34] = self.escrow_type as u8;
    //     data[35..67].copy_from_slice(&self.token_giver_mint);
    //     data[67..99].copy_from_slice(&self.token_take_mint);
    //     data[99..131].copy_from_slice(&self.token_take_amount.to_le_bytes());
    //     data[131] = self.bump;
    //     data
    // }

    // pub fn unpack(data: &[u8]) -> Result<Self, ProgramError> {
    //     let maker = data[0..32]
    //         .try_into()
    //         .map_err(|_| ProgramError::InvalidInstructionData)?;
    //     let seed = data[32..34]
    //         .try_into()
    //         .map_err(|_| ProgramError::InvalidInstructionData)?;
    //     let escrow_type =
    //         EscrowType::try_from(data[34]).map_err(|_| ProgramError::InvalidInstructionData)?;
    //     let token_giver_mint = data[35..67]
    //         .try_into()
    //         .map_err(|_| ProgramError::InvalidInstructionData)?;
    //     let token_giver_amount = u64::from_le_bytes(
    //         data[67..99]
    //             .try_into()
    //             .map_err(|_| ProgramError::InvalidInstructionData)?,
    //     );
    //     let token_take_mint = data[99..131]
    //         .try_into()
    //         .map_err(|_| ProgramError::InvalidInstructionData)?;
    //     let token_take_amount = u64::from_le_bytes(
    //         data[131..163]
    //             .try_into()
    //             .map_err(|_| ProgramError::InvalidInstructionData)?,
    //     );
    //     let bump = data[163];
    //     Ok(Self {
    //         maker,
    //         seed,
    //         escrow_type,
    //         token_giver_mint,
    //         token_giver_amount,
    //         token_take_mint,
    //         token_take_amount,
    //         bump,
    //     })
    // }
}
