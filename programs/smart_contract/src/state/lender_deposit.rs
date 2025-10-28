use crate::errors::ErrorCode;
use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct LenderDeposit {
    pub lender: Pubkey,
    pub amount: u64,          // Amount of SOL our deposit is worth - change over time
    pub last_multiplier: u64, // Value of the global multiplier at our last interaction
    pub bump: u8,
}

impl LenderDeposit {
    // Adjust the amount based on the change in global multiplier
    pub fn adjust_for_lender_multiplier(&mut self, current_lender_multiplier: u128) -> Result<()> {
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
