use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenInterface},
};

use crate::state::{Orderbook, AllAssets};

// Todo: make sure only admin can initialize

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    // Used for the PDA of all_assets - but nothing else
    // Caller is adviced to call it with actually the base asset mint of the market it documents a bit
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

pub fn handler(ctx: Context<Initialize>, start_tick: u64, tick_size: u64) -> Result<()> {
    let all_assets = &mut ctx.accounts.all_assets;
    all_assets.base_asset = ctx.accounts.base_asset.key();
    all_assets.size_assets = 0;
    // all_assets.assets = [Default::default(); MAX_ASSETS];
    all_assets.start_tick = start_tick;
    all_assets.tick_size = tick_size;
    all_assets.global_multiplier = 1;
    all_assets.amount = 0;
    all_assets.last_update_timestamp = Clock::get()?.unix_timestamp;

    Ok(())
}