use pinocchio::{
    account_info::AccountInfo, entrypoint, msg, program_error::ProgramError, pubkey::Pubkey,
    ProgramResult,
};
use pinocchio_pubkey::pubkey;

use crate::instructions::{make_escrow, take_escrow};

pub mod error;
pub mod instructions;
pub mod states;

pub const ID: Pubkey = pubkey!("N9BuK6SmDXHr2jpca1C4WzMhok2wki8sx2osK1sTobc");

entrypoint!(process_instruction);

fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let (descriminator, data) = instruction_data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;
    match descriminator {
        0x01 => {
            msg!("Making escrow");
            make_escrow(program_id, accounts, data)?;
        }
        0x02 => {
            msg!("Taking escrow");
            take_escrow(program_id, accounts, data)?;
        }
        _ => {
            return Err(ProgramError::InvalidInstructionData);
        }
    }
    Ok(())
}
