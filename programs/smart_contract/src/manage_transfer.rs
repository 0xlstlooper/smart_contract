// Is responsible for handling transfer in/out of SPL token
// Sorry, probably not idiomatic Solana code, im from ethereum :)

use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface, TransferChecked, transfer_checked};
use crate::errors::ErrorCode;
use crate::state::AllAssets;

pub fn delta_split_extraction(
    delta_split: &Vec<(u64, i128)>,
    all_assets: &Account<AllAssets>,
) -> ((Vec<u64>, Vec<Pubkey>), (Vec<u64>, Vec<Pubkey>)) {
    // Split into deposits and withdrawals
    let mut deposit_amounts: Vec<u64> = vec![];
    let mut deposit_mints: Vec<Pubkey> = vec![];
    let mut withdraw_amounts: Vec<u64> = vec![];
    let mut withdraw_mints: Vec<Pubkey> = vec![];
    for i in 0..all_assets.size_assets as usize {
        let (tick_index, amount_change) = delta_split[i];
        let mint = all_assets.assets[i].mint;
        if amount_change > 0 {
            deposit_amounts.push(amount_change as u64);
            deposit_mints.push(mint);
        } else if amount_change < 0 {
            withdraw_amounts.push((-amount_change) as u64);
            withdraw_mints.push(mint);
        }
    }
    ((deposit_amounts, deposit_mints), (withdraw_amounts, withdraw_mints))
}

pub fn manage_deposit<'info>(
    // Which transfers to do - determined by the orderbook
    amounts: &Vec<u64>,
    mints:   &Vec<Pubkey>,
    // Parameters given by the user, needs to be checked
    remaining_accounts: &'info [AccountInfo<'info>],
    // Data from the context to do the verification of the parameters above
    payer: &Signer<'info>,                     // Originator of the transfer - do we actually need that? not sure
    vault_authority: &UncheckedAccount<'info>, // Us
    // The rest
    token_program: &mut Interface<'info, TokenInterface>,
    ) -> Result<()>
{
    let account_triplets = remaining_accounts.chunks_exact(3);
    for (i, accounts) in account_triplets.into_iter().enumerate() {
        // Unpack the accounts for this specific asset
        let source_account_info = &accounts[0];
        let vault_asset_info = &accounts[1];
        let mint_asset_info = &accounts[2];

        let source_account = InterfaceAccount::<TokenAccount>::try_from(source_account_info)?;
        let vault_asset = InterfaceAccount::<TokenAccount>::try_from(vault_asset_info)?;
        let mint_asset = InterfaceAccount::<Mint>::try_from(mint_asset_info)?;
        
        // Check mint correspondence - les deux premiers sont pas necessaires? verifier ça et repercuter dans la fonction du bas si besoin
        // require_keys_eq!(source_account.mint, mints[i], ErrorCode::MintMismatch);
        // require_keys_eq!(vault_asset.mint, mints[i], ErrorCode::MintMismatch);
        require_keys_eq!(mint_asset.key(), mints[i], ErrorCode::MintMismatch);

        // Check ownership and authority
        require_keys_eq!(source_account.owner, payer.key(), ErrorCode::OwnerMismatch);
        require_keys_eq!(vault_asset.owner, vault_authority.key(), ErrorCode::VaultOwnerMismatch);

        // --- CPI Call ---
        let cpi_accounts = TransferChecked {
            from: source_account_info.clone(),
            mint: mint_asset_info.clone(),
            to: vault_asset_info.clone(),
            authority: payer.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(token_program.to_account_info(), cpi_accounts);

        transfer_checked(cpi_ctx, amounts[i], mint_asset.decimals)?;
    }
    Ok(())
}

pub fn manage_withdraw<'info>(
    // Which transfers to do - determined by the orderbook
    amounts: &Vec<u64>,
    mints:   &Vec<Pubkey>,
    // Parameters given by the user, needs to be checked
    remaining_accounts: &'info [AccountInfo<'info>],
    // Data from the context to do the verification of the parameters above
    payer: &Signer<'info>,
    vault_authority: &UncheckedAccount<'info>,
    // The rest
    token_program: &mut Interface<'info, TokenInterface>,
    signer_seeds: &[&[&[u8]]],
    ) -> Result<()>
{
    let account_triplets = remaining_accounts.chunks_exact(3);
    for (i, accounts) in account_triplets.into_iter().enumerate() {
        // Unpack the accounts for this specific asset
        let destination_account_info = &accounts[0];
        let vault_asset_info = &accounts[1];
        let mint_asset_info = &accounts[2];

        let destination_account = InterfaceAccount::<TokenAccount>::try_from(destination_account_info)?;
        let vault_asset = InterfaceAccount::<TokenAccount>::try_from(vault_asset_info)?;
        let mint_asset = InterfaceAccount::<Mint>::try_from(mint_asset_info)?;

        // Check mint correspondence
        require_keys_eq!(mint_asset.key(), mints[i], ErrorCode::MintMismatch);

        // Check ownership
        require_keys_eq!(destination_account.owner, payer.key(), ErrorCode::OwnerMismatch);
        require_keys_eq!(vault_asset.owner, vault_authority.key(), ErrorCode::VaultOwnerMismatch);

        // --- CPI Call ---
        let cpi_accounts = TransferChecked {
            from: vault_asset_info.clone(),
            mint: mint_asset_info.clone(),
            to: destination_account_info.clone(),
            authority: vault_authority.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(token_program.to_account_info(), cpi_accounts, signer_seeds);

        transfer_checked(cpi_ctx, amounts[i], mint_asset.decimals)?;
    }
    Ok(())
}