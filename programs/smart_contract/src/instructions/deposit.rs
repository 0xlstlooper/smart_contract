use anchor_lang::{prelude::*, system_program};
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface, TransferChecked, transfer_checked},
};
use crate::errors::ErrorCode;
use crate::state::{AllAssets};
use crate::manage_transfer::*;

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

    /// CHECK: ok
    #[account(
        seeds = [b"vault_authority"],
        bump
    )]
    pub vault_authority: UncheckedAccount<'info>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

// Todo refactoriser pour plus prendre en argument les mints
// Structure of remaining_accounts: for each asset being deposited: [source_account, vault_asset, mint_asset]
pub fn handler<'info>(
    ctx: Context<'_, '_, 'info, 'info, Deposit<'info>>, 
    amount: u64
) -> Result<()> {

    // The amounts we are supposed to deposit for each asset. `true` for deposit.
    let delta_split = ctx.accounts.all_assets.delta_split_lender(amount, true)?;
    let (amounts, mints) = delta_split_extraction(&delta_split, &ctx.accounts.all_assets);

    manage_deposit(
        &amounts,
        &mints,
        &ctx.remaining_accounts,
        &ctx.accounts.payer,
        &ctx.accounts.vault_authority,
        &mut ctx.accounts.token_program,
    )?;

    Ok(())
}