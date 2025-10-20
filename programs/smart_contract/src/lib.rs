use anchor_lang::prelude::*;

mod errors;
mod instructions;
mod state;

use instructions::*;

declare_id!("BDuS1SDwA4NdZRVdUdDLF1r51kuTpYonHUhrfkgEBcfN");

#[program]
pub mod smart_contract {
    use super::*;

    // Initialize each market
    pub fn initialize(ctx: Context<Initialize>, start_tick: u64, tick_size: u64) -> Result<()> {
        instructions::initialize::handler(ctx, start_tick, tick_size)
    }
    // Add an asset
    pub fn add_asset(ctx: Context<AddAsset>, multiplier: u64) -> Result<()> {
        instructions::add_asset::handler(ctx, multiplier)
    }
    // Lenders' section
    /*todo voir si on fait ça ou si on passe un array de tokens
        Flow of use:
        + The frontend of the lender checks which token he is supposed to deposit
        + Then, the lender deposit must be splitted into multiple deposit calls, as
            a single call only deposit a single asset
    */
    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        instructions::deposit::handler(ctx, amount)
    }
    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        instructions::withdraw::handler(ctx, amount)
    }
    // Loopers' section
    pub fn place_bid(ctx: Context<PlaceBid>, slot_index: u64, amount: u64) -> Result<()> {
        instructions::place_bid::handler(ctx, slot_index as usize, amount)
    }
    pub fn remove_bid(ctx: Context<RemoveBid>) -> Result<()> {
        instructions::remove_bid::handler(ctx)
    }
}