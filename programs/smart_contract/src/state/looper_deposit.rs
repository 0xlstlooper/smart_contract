use crate::errors::ErrorCode;
use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct LooperDeposit {
    pub looper: Pubkey,
    pub asset_index: u64,     // Which asset we deposit
    pub slot_index: u64,      // What APY we want to pay
    pub amount: u64,          // Value of the position
    pub last_multiplier: u64, // Value of the global multiplier at our last interaction - to pay our APY
    pub last_decay: u64,      // Value of the low asset decay at our last interaction
    pub bump: u8,
}

impl LooperDeposit {
    // Adjust the amount based on the change in global multiplier
    pub fn adjust_for_looper_multiplier(&mut self, current_looper_multiplier: u128) -> Result<()> {
        let adjusted_amount = (self.amount as u128)
            .checked_mul(current_looper_multiplier)
            .ok_or(ErrorCode::NumErr)?
            .checked_div(self.last_multiplier as u128)
            .ok_or(ErrorCode::NumErr)? as u64;

        self.amount = adjusted_amount;
        self.last_multiplier = current_looper_multiplier as u64;

        Ok(())
    }

    // Adjust the amount based on the change in low asset decay
    pub fn adjust_for_decay(&mut self, current_decay: u128) -> Result<()> {
        let adjusted_amount = (self.amount as u128)
            .checked_mul(current_decay)
            .ok_or(ErrorCode::NumErr)?
            .checked_div(self.last_decay as u128)
            .ok_or(ErrorCode::NumErr)? as u64;

        self.amount = adjusted_amount;
        self.last_decay = current_decay as u64;

        Ok(())
    }
}
