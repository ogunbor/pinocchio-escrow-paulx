use pinocchio::{
    ProgramResult,
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
};
use pinocchio_log::log;

use crate::state::Escrow;

pub fn process_refund_instruction(accounts: &[AccountInfo], _data: &[u8]) -> ProgramResult {
    let [
        maker,
        mint_a,
        maker_ata_a,
        escrow,
        vault,
        _token_program,
        _system_program,
        _remaining @ ..,
    ] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Ensure the maker is a signer, this prevents unauthorized refunds
    if !maker.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Get the escrow state from the escrow account
    let escrow_account = Escrow::from_account_info(escrow);

    // Validate that the escrow belongs to this maker and the mint is correct
    // This ensures we're refunding the correct escrow and tokens
    assert_eq!(escrow_account.maker, *maker.key());
    assert_eq!(escrow_account.mint_x, *mint_a.key());

    // Load the vault account to access token balance and verify ownership
    let vault_account = pinocchio_token::state::TokenAccount::from_account_info(vault)?;

    // Verify that the vault is owned by the escrow PDA
    // This ensures we're operating on the correct vault associated with this escrow
    assert_eq!(vault_account.owner(), escrow.key());

    // Prepare the PDA seeds needed for signing operations
    // The escrow account is a PDA (Program Derived Address) that can sign for transactions
    let bump = [escrow_account.bump.to_le()];
    let seed = [
        Seed::from(b"escrow"),
        Seed::from(maker.key()),
        Seed::from(&bump),
    ];
    let seeds = Signer::from(&seed);

    log!("Refunding tokens to maker");

    // Transfer all tokens from the vault back to the maker's token account
    // The escrow PDA signs this transaction using the computed seeds
    pinocchio_token::instructions::Transfer {
        from: vault,
        to: maker_ata_a,
        authority: escrow,
        amount: vault_account.amount(),
    }
    .invoke_signed(&[seeds.clone()])?;

    // Account not closed yet. Maker might decide to deposit again

    // All operations were successful, complete the refund process
    Ok(())
}
