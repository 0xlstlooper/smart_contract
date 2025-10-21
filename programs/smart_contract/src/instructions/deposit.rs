use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface, TransferChecked, transfer_checked},
};

// Todo - selectionner le bon asset à deposer

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    // The user's token account for the asset being deposited
    #[account(
        mut,
        associated_token::mint = mint_asset,
        associated_token::authority = payer,
    )]
    pub source_account: InterfaceAccount<'info, TokenAccount>,
    
    // The program's vault for the asset
    #[account(
        mut,
        associated_token::mint = mint_asset,
        associated_token::authority = vault_authority,
    )]
    pub vault_asset: InterfaceAccount<'info, TokenAccount>,

    // The mint of the asset
    pub mint_asset: InterfaceAccount<'info, Mint>,

    // The PDA authority for the vault
    #[account(
        seeds = [b"vault_authority"],
        bump
    )]
    /// CHECK: This is the vault authority PDA.
    pub vault_authority: AccountInfo<'info>,

    // System Programs
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Deposit>, amount: u64) -> Result<()> {
    // Transfer tokens from user's account to the vault
    let cpi_accounts = TransferChecked {
        from: ctx.accounts.source_account.to_account_info(),
        mint: ctx.accounts.mint_asset.to_account_info(),
        to: ctx.accounts.vault_asset.to_account_info(),
        authority: ctx.accounts.payer.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    transfer_checked(cpi_ctx, amount, ctx.accounts.mint_asset.decimals)?;

    Ok(())
}