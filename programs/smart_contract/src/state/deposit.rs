use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct LenderDeposit {
    pub lender: Pubkey,
    pub amount: u64,           // Amount of SOL deposited
    pub start_multiplier: u64, // Value of the global multiplier at the time of deposit
    pub bump: u8,
}