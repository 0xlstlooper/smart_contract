use anchor_lang::prelude::*;
use crate::errors::ErrorCode;

pub const MAX_ASSETS: u64 = 2;
pub const ORDERBOOK_SIZE: usize = 10;

/*
The first index is the lowest APY offered - this is start_tick
The last index is the highest APY offered - index i corresponds to start_tick + i * tick_size
*/
#[account]
#[derive(InitSpace)]
pub struct Orderbook {
    pub slots: [u64; ORDERBOOK_SIZE],
    pub best_idx: u64, // index of the current best tick which has some liquidity
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct AssetInfo {
    pub mint: Pubkey,
    pub vault: Pubkey,
    pub multiplier: u64, // Related to the LTV of the asset
    pub orderbook: Orderbook,
}

#[account]
#[derive(InitSpace)]
pub struct AllAssets {
    pub assets: [AssetInfo; MAX_ASSETS as usize],
    pub last_idx: u64,
    // Information shared accross all the orderbooks
    pub start_tick: u64,
    pub tick_size: u64,
}

impl AllAssets {
    // Returns (best_tick, index of the related asset, liquidity of this tick of this asset)
    // Maximize only the tick, not the liquidity - for the same tick, it does not matter which asset we choose
    pub fn current_best_apy(&self) -> Result<(u64, usize, u64)> {
        let mut best_offer: Option<(u64, usize, u64)> = None;

        // Iterate through all the assets that have been added
        for i in 0..self.last_idx as usize {
            let asset = &self.assets[i];
            let orderbook = &asset.orderbook;
            
            // Get the best tick index and the liquidity available at that tick for the current asset
            let best_tick_idx = orderbook.best_idx as usize;
            let liquidity = orderbook.slots[best_tick_idx as usize];

            // We only consider this asset if there is liquidity at its best tick
            if liquidity > 0 {
                // Calculate the APY for the current asset's best tick
                let current_apy = self.start_tick + (best_tick_idx as u64) * self.tick_size;

                match best_offer {
                    Some((best_apy_so_far, _, _)) => {
                        // If the current asset's APY is better than the best we've seen, update it
                        if current_apy > best_apy_so_far {
                            best_offer = Some((current_apy, i, liquidity));
                        }
                    }
                    None => {
                        // If this is the first asset with liquidity we've found, it's the best by default
                        best_offer = Some((current_apy, i, liquidity));
                    }
                }
            }
        }

        // If a best_offer was found, return it, otherwise return an error
        best_offer.ok_or_else(|| error!(ErrorCode::NoLiquidityAvailable))
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_double() {
        // let s = MyStruct { value: 5 };
        // assert_eq!(s.double(), 10);
    }
}