use anchor_lang::prelude::*;

mod errors;
mod instructions;
mod state;
mod manage_transfer;

use instructions::*;

declare_id!("BDuS1SDwA4NdZRVdUdDLF1r51kuTpYonHUhrfkgEBcfN");

#[program]
pub mod smart_contract {
    use super::*;

    // Initialize each market - by admin (?)
    pub fn initialize(ctx: Context<Initialize>, start_tick: u64, tick_size: u64) -> Result<()> {
        instructions::initialize::handler(ctx, start_tick, tick_size)
    }
    // Add an asset to a market - by admin
    pub fn add_asset(ctx: Context<AddAsset>, multiplier: u64) -> Result<()> {
        instructions::add_asset::handler(ctx, multiplier)
    }
    // Lenders' section
    /*
        Flow of use:
        + The frontend of the lender checks which tokenS and amounts of them he is supposed to deposit
        + Then, with a single call to this function he deposits all the tokens at once
        + Funny enough, because the frontend do the swap separately, if the user gets frontrunned,
            the transaction will fail because the required tokens to be depositted will be different
            --> No slippage parameter for this code, if you get frontrunned, you need to do a new tx
            (Although the frontend has a slippage parameter when doing the swaps to get the required tokens,
            but we use exact output, so the user will always get the exact amount of tokens needed for the deposit,
            even if what he pays will differ because of slippage)
        // The amount parameter allows the code to compute how much of each asset to deposit
    */
    pub fn deposit<'info>(ctx: Context<'_, '_, 'info, 'info, Deposit<'info>>, amount: u64) -> Result<()> {
        instructions::deposit::handler(ctx, amount)
    }
    // Same - when frontrunned, the user will get a different split of tokens
    pub fn withdraw<'info>(ctx: Context<'_, '_, 'info, 'info, Withdraw<'info>>, amount: u64) -> Result<()> {
        instructions::withdraw::handler(ctx, amount)
    }
    // Loopers' section
    /*
        Flow of use: -- todo factoriser les transfers dans une autre fonction pcq ces fonctions font ça aussi

    */
    pub fn place_bid(ctx: Context<PlaceBid>, slot_index: u64, amount: u64) -> Result<()> {
        instructions::place_bid::handler(ctx, slot_index as usize, amount)
    }
    // Todo change bid?
    // -- A looper function
    pub fn remove_bid(ctx: Context<RemoveBid>) -> Result<()> {
        instructions::remove_bid::handler(ctx)
    }
}