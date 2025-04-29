use pinocchio::{
    ProgramResult,
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    pubkey::find_program_address,
};

use crate::state::Escrow;

pub fn process_take_instruction(accounts: &[AccountInfo], _data: &[u8]) -> ProgramResult {
    // Unpack all required accounts for the take operation
    let [
        maker,
        mint_x,
        mint_y,
        taker_ata_x,
        maker_ata_y,
        vault_x,
        vault_y,
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
    // This prevents trading with incorrect tokens
    assert_eq!(escrow_account.mint_x, *mint_x.key());
    assert_eq!(escrow_account.mint_y, *mint_y.key());

    // Load the vault account to access its token balance
    let vault_account_x = pinocchio_token::state::TokenAccount::from_account_info(vault_x)?;
    // Load the vault account to access its token balance
    let vault_account_y = pinocchio_token::state::TokenAccount::from_account_info(vault_y)?;

    // Verify the escrow account is a valid PDA with the expected seeds
    // This ensures we're operating on a legitimate escrow created by our program
    let seed = [(b"escrow"), maker.key().as_slice(), &[escrow_account.bump]];
    let seeds = &seed[..];
    let escrow_pda = find_program_address(seeds, &crate::id()).0;
    assert_eq!(*escrow.key(), escrow_pda);

    // First leg of the trade: tokens from vault_x to taker_ata
    let bump = [escrow_account.bump.to_le()];
    let seed = [
        Seed::from(b"escrow"),
        Seed::from(maker.key()),
        Seed::from(&bump),
    ];
    let seeds_one = Signer::from(&seed);

    pinocchio_token::instructions::Transfer {
        from: vault_x,
        to: taker_ata_x,
        authority: escrow,
        amount: vault_account_x.amount(),
    }
    .invoke_signed(&[seeds_one.clone()])?;

    // 2nd leg of the trade: tokens from vault_y to maker_ata
    let bump_y = [escrow_account.vault_y_bump.to_le()];
    let seed = [
        Seed::from(b"vault_y"),
        Seed::from(maker.key()),
        Seed::from(mint_y.key()),
        Seed::from(&bump_y),
    ];
    let seeds_two = Signer::from(&seed);

    pinocchio_token::instructions::Transfer {
        from: vault_y,
        to: maker_ata_y,
        authority: escrow,
        amount: vault_account_y.amount(),
    }
    .invoke_signed(&[seeds_two.clone()])?;

    // Close the vault account and return the rent to the maker
    // The maker paid to create this account, so they receive the lamports
    for (vault, seeds) in [(vault_x, seeds_one), (vault_y, seeds_two)] {
        pinocchio_token::instructions::CloseAccount {
            account: vault,
            destination: maker,
            authority: escrow,
        }
        .invoke_signed(&[seeds])?;
    }

    // Manually close the escrow account and return rent to the maker
    // This completes the trade by cleaning up all accounts
    unsafe {
        *maker.borrow_mut_lamports_unchecked() += *escrow.borrow_lamports_unchecked();
        *escrow.borrow_mut_lamports_unchecked() = 0
    };

    Ok(())
}
