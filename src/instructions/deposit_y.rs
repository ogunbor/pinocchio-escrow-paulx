use pinocchio::{ProgramResult, account_info::AccountInfo, pubkey::find_program_address};

use crate::state::Escrow;

pub fn process_deposit_y_instruction(accounts: &[AccountInfo], _data: &[u8]) -> ProgramResult {
    let [
        taker,
        maker,
        mint_y,
        vault_y,
        taker_ata_y,
        escrow,
        _token_program,
        _system_program,
        _remaining @ ..,
    ] = accounts
    else {
        return Err(pinocchio::program_error::ProgramError::NotEnoughAccountKeys);
    };

    // Access the escrow data to verify trade parameters
    let escrow_account = Escrow::from_account_info(escrow);

    // Verify that the provided token mints match what's stored in the escrow
    assert_eq!(escrow_account.mint_y, *mint_y.key());

    // Verify that the vault's key in the escrow matches the provided
    assert_eq!(escrow_account.vault_y, *vault_y.key());

    // Verify the escrow account is a valid PDA with the expected seeds
    // This ensures we're operating on a legitimate escrow created by our program
    let seed = [(b"escrow"), maker.key().as_slice(), &[escrow_account.bump]];
    let seeds = &seed[..];
    let escrow_pda = find_program_address(seeds, &crate::id()).0;
    assert_eq!(*escrow.key(), escrow_pda);

    // The taker pays the requested amount of token Y directly to the maker
    pinocchio_token::instructions::Transfer {
        from: taker_ata_y,
        to: vault_y,
        authority: taker,
        amount: escrow_account.amount,
    }
    .invoke()?;

    Ok(())
}
