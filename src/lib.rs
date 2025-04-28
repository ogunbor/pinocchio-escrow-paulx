use pinocchio::{
    ProgramResult, account_info::AccountInfo, entrypoint, program_error::ProgramError,
    pubkey::Pubkey,
};

pub mod instructions;
pub mod state;

use pinocchio_pubkey::declare_id;
declare_id!("FFEfkkRGnefuA2TxPSGDbjV9cgBrWM3EtspEuxtYFkA");

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    Ok(())
}
