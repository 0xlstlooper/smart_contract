use anchor_lang::{prelude::*, system_program};
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};
use crate::errors::ErrorCode;
use crate::state::{AllAssets};
use crate::manage_transfer::*;

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"all_assets", all_assets.base_asset.key().as_ref()],
        bump,
    )]
    pub all_assets: Account<'info, AllAssets>,

    /// CHECK: This is the vault's authority, a PDA. Its seeds are verified in the transfer.
    #[account(
        seeds = [b"vault_authority"],
        bump
    )]
    pub vault_authority: UncheckedAccount<'info>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

// Structure of remaining_accounts: for each asset being withdrawn: [destination_account, vault_asset, mint_asset]
pub fn handler<'info>(
    ctx: Context<'_, '_, 'info, 'info, Withdraw<'info>>,
    amount: u64
) -> Result<()> {

    // The amounts we are supposed to withdraw for each asset. `false` for withdrawal.
    let delta_split = ctx.accounts.all_assets.delta_split_lender(amount, false)?;
    let (amounts, mints) = delta_split_extraction(&delta_split, &ctx.accounts.all_assets);
    
    // Prepare the signer seeds for the vault authority PDA
    let bump = ctx.bumps.vault_authority;
    let signer_seeds = &[&b"vault_authority"[..], &[bump]];
    let signer = &[&signer_seeds[..]];

    manage_withdraw(
        &amounts,
        &mints,
        &ctx.remaining_accounts,
        &ctx.accounts.payer,
        &ctx.accounts.vault_authority,
        &mut ctx.accounts.token_program,
        signer,
    )?;

    Ok(())
}