use anchor_lang::{prelude::*, system_program};
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface, TransferChecked, transfer_checked},
};
use crate::errors::ErrorCode;
use crate::state::{AllAssets};

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

    /// CHECK: This is the vault authority PDA, its seeds are verified.
    #[account(
        seeds = [b"vault_authority"],
        bump
    )]
    pub vault_authority: AccountInfo<'info>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

// remaining_accounts: for each asset being deposited:
// [source_account, vault_asset, mint_asset]
pub fn handler<'info>(
    ctx: Context<'_, '_, 'info, 'info, Deposit<'info>>, 
    amount: u64
) -> Result<()> {
    
    // Format of the structure: [source_account, vault_asset, mint_asset]
    let account_triplets = ctx.remaining_accounts.chunks_exact(3);

    // The amounts we are supposed to deposit for each asset. `true` for deposit.
    let amounts = ctx.accounts.all_assets.delta_split_sol(amount, true)?;

    for (amount, accounts) in amounts.iter().zip(account_triplets) {
        // Unpack the accounts for this specific asset
        let source_account_info = &accounts[0];
        let vault_asset_info = &accounts[1];
        let mint_asset_info = &accounts[2];

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

        transfer_checked(cpi_ctx, amount.1, mint_asset.decimals)?;
    }

    Ok(())
}