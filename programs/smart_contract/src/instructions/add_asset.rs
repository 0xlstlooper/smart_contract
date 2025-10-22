use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{ Mint, TokenAccount, TokenInterface, TransferChecked, transfer_checked};

use crate::state::{Config, Orderbook, AllAssets, MAX_ASSETS};
use crate::errors::ErrorCode;

// Todo ajouter que le payer est forcement l'admin

#[derive(Accounts)]
pub struct AddAsset<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"allassets"],
        bump,
    )]
    pub allassets: Account<'info, AllAssets>,
    // Vault authority
    #[account(
        seeds = [b"vault_authority"],
        bump
    )]
    pub vault_authority: SystemAccount<'info>,
    // Mint
    #[account(
        mint::token_program = token_program
    )]
    pub mint_asset: InterfaceAccount<'info, Mint>,
    // Vault
    #[account(
        init,
        payer = payer,
        associated_token::mint = mint_asset,
        associated_token::authority = vault_authority,
    )]
    pub vault_asset: InterfaceAccount<'info, TokenAccount>,
    // Orderbook
    #[account(
        init,
        payer = payer,
        seeds = [b"orderbook", mint_asset.key().as_ref()],
        bump,
        space = 8 + Orderbook::INIT_SPACE,
    )]
    pub orderbook: Account<'info, Orderbook>,
    // Rest
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<AddAsset>, multiplier: u64) -> Result<()> {
    let idx = ctx.accounts.allassets.size_assets;
    require!(idx < MAX_ASSETS, ErrorCode::AllAssetsIsFull);
    ctx.accounts.allassets.size_assets += 1;
    let allassets = &mut ctx.accounts.allassets.assets;
    let asset = &mut allassets[idx as usize];
    // Verify asset is not initialized yet
    require!(asset.multiplier == 0, ErrorCode::AssetAlreadyInitialized);
    asset.mint = ctx.accounts.mint_asset.key();
    asset.vault = ctx.accounts.vault_asset.key();
    asset.multiplier = multiplier;  

    Ok(())
}