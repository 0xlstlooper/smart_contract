use anchor_lang::{prelude::*, system_program};
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface, TransferChecked, transfer_checked},
};
use crate::errors::ErrorCode;
use crate::state::{AllAssets};

// Todo, check that the assets deposited are the one excepted from the datastructure

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"all_assets", all_assets.base_asset.key().as_ref()],
        bump,
    )]
    pub all_assets: Account<'info, AllAssets>,

    #[account(
        seeds = [b"vault_authority"],
        bump
    )]
    /// CHECK: This is the vault authority PDA, its seeds are verified.
    pub vault_authority: AccountInfo<'info>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler<'info>(
    ctx: Context<'_, '_, 'info, 'info, Deposit<'info>>, 
    amounts: Vec<u64>
) -> Result<()> {
    
    // Use chunks_exact to get account triplets. This is safer and cleaner.
    let account_triplets = ctx.remaining_accounts.chunks_exact(3);

    // Ensure the number of amounts matches the number of asset triplets
    require!(amounts.len() == account_triplets.len(), ErrorCode::InvalidInputLength);

    // The amounts we are supposed to deposit for each asset
    // let amounts = ctx.all_assets

    // Zip the amounts and account triplets together to process them in pairs
    for (amount, accounts) in amounts.iter().zip(account_triplets) {
        // Unpack the accounts for this specific asset
        let source_account_info = &accounts[0];
        let vault_asset_info = &accounts[1];
        let mint_asset_info = &accounts[2];

        // --- MANUAL VALIDATION ---
        let source_account = InterfaceAccount::<TokenAccount>::try_from(source_account_info)?;
        let vault_asset = InterfaceAccount::<TokenAccount>::try_from(vault_asset_info)?;
        let mint_asset = InterfaceAccount::<Mint>::try_from(mint_asset_info)?;

        // Check mint correspondence
        require_keys_eq!(source_account.mint, mint_asset.key(), ErrorCode::MintMismatch);
        require_keys_eq!(vault_asset.mint, mint_asset.key(), ErrorCode::MintMismatch);

        // Check ownership and authority
        require_keys_eq!(source_account.owner, ctx.accounts.payer.key(), ErrorCode::OwnerMismatch);
        require_keys_eq!(vault_asset.owner, ctx.accounts.vault_authority.key(), ErrorCode::VaultOwnerMismatch);

        // --- CPI Call ---
        let cpi_accounts = TransferChecked {
            from: source_account_info.clone(),
            mint: mint_asset_info.clone(),
            to: vault_asset_info.clone(),
            authority: ctx.accounts.payer.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        transfer_checked(cpi_ctx, *amount, mint_asset.decimals)?;
    }

    Ok(())
}