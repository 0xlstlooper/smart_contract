use anchor_lang::prelude::*;

pub mod all_assets; 
pub use all_assets::*;

pub mod deposit;
pub use deposit::*;

#[account]
pub struct Bid {
    pub user: Pubkey,
    pub slot_index: u32,
    pub jitosol_amount: u64,
    pub asset_amount: u64,
}