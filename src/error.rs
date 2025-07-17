use pinocchio::program_error::ProgramError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EscrowErrorCode {
    InvalidMaker,
    EscrowAlreadyExists,
    TokenAccountAlreadyExists,
    PdaMismatch,
    InvalidTokenOwner,
    InvalidMakerTokenAccount,
    InvalidTokenMint,
    MintMismatch,
    InvalidEscrowType,
    InsufficientFunds,
}

impl From<EscrowErrorCode> for ProgramError {
    fn from(e: EscrowErrorCode) -> Self {
        ProgramError::Custom(e as u32)
    }
}
