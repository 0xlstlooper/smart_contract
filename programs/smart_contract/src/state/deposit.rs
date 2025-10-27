use anchor_lang::prelude::*;
use crate::errors::ErrorCode;

#[account]
#[derive(InitSpace)]
pub struct LenderDeposit {
    pub lender: Pubkey,
    pub amount: u64,          // Amount of SOL our deposit is worth - change over time
    pub last_multiplier: u64, // Value of the global multiplier at our last interaction
    pub bump: u8,
}

impl LenderDeposit {
    pub fn adjust_for_lender_multiplier(&mut self, current_lender_multiplier: u128) -> Result<()> {
        // Adjust the amount based on the change in global multiplier
        let adjusted_amount = (self.amount as u128)
            .checked_mul(current_lender_multiplier)
            .ok_or(ErrorCode::NumErr)?
            .checked_div(self.last_multiplier as u128)
            .ok_or(ErrorCode::NumErr)? as u64;

        self.amount = adjusted_amount;
        self.last_multiplier = current_lender_multiplier as u64;

        Ok(())
    }
}

#[account]
#[derive(InitSpace)]
pub struct LooperDeposit {
    pub looper: Pubkey,
    pub asset_index: u64, // Which asset we deposit
    pub slot_index: u64,  // What APY we want to pay
    pub amount: u64,          // Value of the position
    pub last_multiplier: u64, // Value of the global multiplier at our last interaction - to pay our APY
    pub bump: u8,
}