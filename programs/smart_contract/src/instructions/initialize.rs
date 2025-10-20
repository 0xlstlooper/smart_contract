use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token},
};

use crate::state::{Config, Orderbook, AllAssets};

// Todo yaura plusieurs orderbooks, todo la yen a qu'un

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    // Configs/AllAssets
    #[account(
        init,
        payer = payer,
        seeds = [b"config"],
        bump,
        space = 8 + Config::INIT_SPACE,
    )]
    pub config: Account<'info, Config>,
    #[account(
        init,
        payer = payer,
        seeds = [b"allassets"],
        bump,
        space = 8 + AllAssets::INIT_SPACE,
    )]
    pub allassets: Account<'info, AllAssets>,
    // Rest
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Initialize>, start_tick: u64, tick_size: u64) -> Result<()> {
    let config = &mut ctx.accounts.config;
    config.admin = ctx.accounts.payer.key();
    let allassets = &mut ctx.accounts.allassets;
    allassets.size_assets = 0;
    allassets.start_tick = start_tick;
    allassets.tick_size = tick_size;
    // config.jitosol_mint = ctx.accounts.jitosol_mint.key();
    // let orderbook = &mut ctx.accounts.orderbook;

    Ok(())
}