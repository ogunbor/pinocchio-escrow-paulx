pub mod deposit_y;
pub mod make;
pub mod refund;
pub mod take;
pub use deposit_y::*;
pub use make::*;
pub use refund::*;
pub use take::*;

use pinocchio::program_error::ProgramError;

pub enum EscrowInstructions {
    Make = 0,
    DepositY = 1,
    Take = 2,
    Refund = 3,
}

impl TryFrom<u8> for EscrowInstructions {
    type Error = ProgramError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(EscrowInstructions::Make),
            1 => Ok(EscrowInstructions::DepositY),
            2 => Ok(EscrowInstructions::Take),
            3 => Ok(EscrowInstructions::Refund),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}
