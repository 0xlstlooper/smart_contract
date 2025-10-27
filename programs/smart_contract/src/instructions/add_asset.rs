use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{ Mint, TokenAccount, TokenInterface, TransferChecked, transfer_checked};

use crate::state::{Orderbook, AllAssets, MAX_ASSETS};
use crate::errors::ErrorCode;

// Todo faire en sorte que le payer est forcement l'admin du code et que n'importe quel jacky puisse pas call ce truc.

#[derive(Accounts)]
pub struct AddAsset<'info> {
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

    // Programs
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<AddAsset>, leverage: u64) -> Result<()> {
    let idx = ctx.accounts.all_assets.size_assets;
    require!(idx < MAX_ASSETS, ErrorCode::AllAssetsIsFull);
    ctx.accounts.all_assets.size_assets += 1;
    let all_assets = &mut ctx.accounts.all_assets.assets;
    let asset = &mut all_assets[idx as usize];
    // Verify asset is not initialized yet
    // Todo cette ligne sert a que dalle
    require!(asset.leverage == 0, ErrorCode::AssetAlreadyInitialized);
    asset.mint = ctx.accounts.mint_asset.key();
    asset.vault = ctx.accounts.vault_asset.key();
    asset.leverage = leverage;

    Ok(())
}