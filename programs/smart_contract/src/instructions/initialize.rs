use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenInterface},
};

use crate::constants::*;
use crate::state::{AllAssets, Orderbook};

// Todo: make sure only admin can initialize

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    // This parameter, base_asset, is used only for the PDA of all_assets - nothing else
    pub base_asset: InterfaceAccount<'info, Mint>,

    #[account(
        init,
        payer = payer,
        seeds = [b"all_assets", base_asset.key().as_ref()],
        bump,
        space = 8 + AllAssets::INIT_SPACE,
    )]
    pub all_assets: Account<'info, AllAssets>,

    // Rest
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Initialize>, start_apy: u64, apy_tick: u64) -> Result<()> {
    let all_assets = &mut ctx.accounts.all_assets;
    all_assets.base_asset = ctx.accounts.base_asset.key();
    all_assets.size_assets = 0;
    // all_assets.assets = [Default::default(); MAX_ASSETS];
    all_assets.start_apy = start_apy;
    all_assets.apy_tick = apy_tick;
    all_assets.amount = 0;
    all_assets.lender_multiplier = START_MULTIPLIER_VALUE;
    all_assets.last_update_timestamp = Clock::get()?.unix_timestamp;

    Ok(())
}
