use pinocchio::{
    ProgramResult,
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    pubkey,
    pubkey::find_program_address,
    sysvars::{Sysvar, rent::Rent},
};
use pinocchio_log::log;
use pinocchio_token::state::TokenAccount;

use crate::state::Escrow;

pub fn process_make_instruction(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    // Unpack the required accounts from the accounts array2
    let [
        maker,
        mint_x,
        mint_y,
        maker_ata,
        vault,
        escrow,
        _system_program,
        _token_program,
        _remaining @ ..,
    ] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if data.len() < 17 {
        return Err(ProgramError::InvalidInstructionData);
    }
    // Extract the bump seed from instruction data and prepare seeds for PDA validation
    let bump = unsafe { *(data.as_ptr() as *const u8) }.to_le_bytes();
    let seed = [(b"escrow"), maker.key().as_slice(), bump.as_ref()];
    let seeds = &seed[..];

    // Derive the expected PDA and verify it matches the provided escrow account
    // This ensures the escrow account is derived correctly for this maker and trade
    let pda = pubkey::checked_create_program_address(seeds, &crate::id()).unwrap();
    assert_eq!(&pda, escrow.key());

    if escrow.data_is_empty() {
        unsafe {
            // Verify that the provided mint accounts are legitimate SPL token mints
            assert_eq!(mint_x.owner(), &pinocchio_token::ID);
            assert_eq!(mint_y.owner(), &pinocchio_token::ID);

            // Verify that the vault is owned by the escrow account (for later token operations)
            assert!(
                TokenAccount::from_account_info_unchecked(vault)
                    .unwrap()
                    .owner()
                    == escrow.key()
            );

            // Check if the escrow account needs to be created (first-time initialization)
            if escrow.owner() != &crate::id() {
                log!("Creating Escrow Account");
                let seed = [
                    Seed::from(b"escrow"),
                    Seed::from(maker.key()),
                    Seed::from(&bump),
                ];
                let seeds = Signer::from(&seed);

                // Create the escrow account with enough space for the state data
                // This account will store all the relevant trade details
                pinocchio_system::instructions::CreateAccount {
                    from: maker,
                    to: escrow,
                    lamports: Rent::get()?.minimum_balance(Escrow::SIZE),
                    space: Escrow::SIZE as u64,
                    owner: &crate::id(),
                }
                .invoke_signed(&[seeds])?;

                // Initialize the escrow data with the trade parameters
                let escrow_account = Escrow::from_account_info(&escrow);

                escrow_account.maker = *maker.key();
                escrow_account.mint_x = *mint_x.key();
                escrow_account.mint_y = *mint_y.key();
                escrow_account.amount = *(data.as_ptr().add(1) as *const u64); // Amount of token Y to receive
                escrow_account.bump = *data.as_ptr(); // Store bump for future PDA derivation
                let amount = *(data.as_ptr().add(1 + 8) as *const u64); // amount of token X to deposit in the vault

                // Derive vault_y PDA
                let (vault_y, _vault_y_bump) = find_program_address(
                    &[b"vault_y", maker.key().as_slice(), mint_y.key().as_slice()],
                    &crate::id(),
                );

                // Set vault_y into escrow
                escrow_account.vault_y = vault_y;
                escrow_account.vault_y_bump = _vault_y_bump;

                log!("Amount to deposit: {}", amount);

                // Transfer the offered tokens from maker's account to the vault
                // These tokens will be locked until someone takes the trade or the maker refunds
                pinocchio_token::instructions::Transfer {
                    from: maker_ata,
                    to: vault,
                    authority: maker,
                    amount, // Amount of token X to deposit
                }
                .invoke()?;
            } else {
                return Err(ProgramError::AccountAlreadyInitialized);
            }
        }
    }

    // Escrow successfully created and tokens deposited
    Ok(())
}
