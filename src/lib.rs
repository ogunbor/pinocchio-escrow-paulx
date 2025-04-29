use instructions::EscrowInstructions;
use pinocchio::{
    ProgramResult, account_info::AccountInfo, entrypoint, program_error::ProgramError,
    pubkey::Pubkey,
};

pub mod instructions;
pub mod state;

entrypoint!(process_instruction);

use pinocchio_pubkey::declare_id;
declare_id!("FFEfkkRGnefuA2TxPSGDbjV9cgBrWM3EtspEuxtYFkA");

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    assert_eq!(program_id, &id());

    let (discriminator, data) = instruction_data
        .split_first()
        .ok_or(ProgramError::InvalidAccountData)?;

    match EscrowInstructions::try_from(*discriminator)? {
        EscrowInstructions::Make => instructions::process_make_instruction(accounts, data)?,
        EscrowInstructions::DepositY => {
            instructions::process_deposit_y_instruction(accounts, data)?
        }
        EscrowInstructions::Take => instructions::process_take_instruction(accounts, data)?,
        EscrowInstructions::Refund => instructions::process_refund_instruction(accounts, data)?,
    }
    Ok(())
}
