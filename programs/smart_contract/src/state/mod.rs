use anchor_lang::prelude::*;

pub mod all_assets; 
pub use all_assets::*;

#[account]
#[derive(InitSpace)]
pub struct Config {
    pub admin: Pubkey,
}

#[account]
#[derive(InitSpace)]
pub struct LenderDeposit {
    pub lender: Pubkey,
    pub mint_asset: Pubkey,
    pub slot_index: u64,
    pub amount: u64,
    pub bump: u8,
}

#[account]
pub struct Bid {
    pub user: Pubkey,
    pub slot_index: u32,
    pub jitosol_amount: u64,
    pub asset_amount: u64,
}