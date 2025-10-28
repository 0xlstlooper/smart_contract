use anchor_lang::prelude::*;

mod errors;
mod instructions;
mod state;
mod utility;
mod constants;

use instructions::*;

declare_id!("BDuS1SDwA4NdZRVdUdDLF1r51kuTpYonHUhrfkgEBcfN");

#[program]
pub mod smart_contract {
    use super::*;

    // Initialize each market - only by admin
    pub fn initialize(ctx: Context<Initialize>, start_tick: u64, tick_size: u64) -> Result<()> {
        instructions::initialize::handler(ctx, start_tick, tick_size)
    }
    // Add an asset to a market - only by admin
    pub fn add_asset(ctx: Context<AddAsset>, leverage: u64) -> Result<()> {
        instructions::add_asset::handler(ctx, leverage)
    }
    // Lenders' deposit/withdraw
    pub fn deposit<'info>(ctx: Context<'_, '_, 'info, 'info, Deposit<'info>>, amount: u64) -> Result<()> {
        instructions::deposit::handler(ctx, amount)
    }
    pub fn withdraw<'info>(ctx: Context<'_, '_, 'info, 'info, Withdraw<'info>>, amount: u64) -> Result<()> {
        instructions::withdraw::handler(ctx, amount)
    }
    // Loopers' place/remove bid
    pub fn place_bid<'info>(ctx: Context<'_, '_, 'info, 'info, PlaceBid<'info>>, asset_index: u64, slot_index: u64, amount: u64) -> Result<()> {
        instructions::place_bid::handler(ctx, asset_index as usize, slot_index as usize, amount)
    }
    pub fn remove_bid<'info>(ctx: Context<'_, '_, 'info, 'info, RemoveBid<'info>>, asset_index: u64, slot_index: u64) -> Result<()> {
        instructions::remove_bid::handler(ctx, asset_index as usize, slot_index as usize)
    }
    // Called by anyone to liquidate a position
    pub fn liquidate_bid<'info>(ctx: Context<'_, '_, 'info, 'info, LiquidateBid<'info>>, owner: Pubkey, asset_index: u64, slot_index: u64) -> Result<()> {
        instructions::liquidate_bid::handler(ctx, owner, asset_index as usize, slot_index as usize)
    }
}