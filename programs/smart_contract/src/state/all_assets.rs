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
    // Identifier of the market
    pub base_asset: Pubkey, // Mint of the base asset - never used in the logic per se but just to have it recorded somewhere because of the PDA and we want to have one unique market per asset
    // List of all assets
    pub size_assets: u64,
    pub assets: [AssetInfo; MAX_ASSETS as usize], // Array filled only until size_assets - todo le transformer en vec
    // Information shared across all the orderbooks
    pub start_tick: u64,
    pub tick_size: u64,
    // Total amount of SOL deposited by lenders in this market
    pub amount: u64,
}

/* Invariant of the structure:
todo

*/

impl AllAssets {
    // Returns (best_tick, index of the related asset, liquidity of this tick of this asset)
    // Maximize only the tick, not the liquidity - for the same tick, it does not matter which asset we choose
    pub fn current_best_apy(&self) -> Result<(u64, usize, u64)> {
        let mut best_offer: Option<(u64, usize, u64)> = None;

        // Iterate through all the assets that have been added
        for i in 0..self.size_assets as usize {
            let asset = &self.assets[i];
            let orderbook = &asset.orderbook;
            
            // Iterate backwards through the orderbook slots to find the highest APY with liquidity for this asset
            for j in (0..ORDERBOOK_SIZE).rev() {
                let liquidity = orderbook.slots[j];

                // If we find the highest tick with liquidity for this asset, we can check if it's the new best overall
                if liquidity > 0 {
                    let current_apy = self.start_tick + (j as u64) * self.tick_size;

                    match best_offer {
                        Some((best_apy_so_far, _, _)) => {
                            // If the current asset's best APY is better than the best we've seen, update it
                            if current_apy > best_apy_so_far {
                                best_offer = Some((current_apy, i, liquidity));
                            }
                        }
                        None => {
                            // If this is the first offer with liquidity we've found, it's the best by default
                            best_offer = Some((current_apy, i, liquidity));
                        }
                    }
                    
                    // Once we've found the best tick for this asset, we can move to the next asset
                    break;
                }
            }
        }

        // If a best_offer was found, return it, otherwise return an error
        best_offer.ok_or_else(|| error!(ErrorCode::NoLiquidityAvailable))
    }

    // Useless in itself in the code, only delta_split_* are used
    // Takes the amount from self.amount in SOL to split on the orderbook by selecting the best APY available iteratively
    // Example: if the orderbook has 500 at 120% and 300 at 130%, and we want to split 600,
    //   we will take 500 at 120% and 100 at 130%
    // Need to return an array of (tick_index, amount), which represents for each assets
    //   upon which tick we selected their liquidity, and what amount we took from it
    // So: the sum all amounts must be equal to the input amount
    // Result is a vector of size all_assets.size_assets
    // @Audrey
    pub fn split_lenders_sol(&self) -> Result<Vec<(u64, u64)>> {
        let mut result: Vec<(u64, u64)> = vec![(0, 0); self.size_assets as usize];
        Ok(result)
    }

    /*
    self.amount is the amount of SOL already splitted, and then delta is the change (+ or -)
    We return what changes needs to be applied to each asset's split,
    aka changes that needs to be done by the smart contract in deposit/withdraw function so the resulting split is correct after a deposit/withdraw
    Essentially, split_lenders_sol(amount = start_amount + delta) = split_lenders_sol(amount = start_amount) + delta_split_lender(amount = start_amount, delta, true)
    So after a deposit/withdraw, we apply the changes returned by delta_split_lender to the current split to get the new split
    result[i] = (tick_index, amount) for asset i
    */
    // Result is a vector of size all_assets.size_assets
    // @Audrey
    pub fn delta_split_lender(&self, delta: u64, sign: bool) -> Result<Vec<(u64, u64)>> {
        let mut result: Vec<(u64, u64)> = vec![(0, 0); self.size_assets as usize];
        Ok(result)
    }

    /* Basically, same as above, but for loopers
    Whats the new repartition once we apply a change of `delta` to asset of index `index`, at slot `slot`
    */
    pub fn delta_split_looper(&self, index: u64, slot: u64, delta: u64, sign: bool) -> Result<Vec<(u64, u64)>> {
        let mut result: Vec<(u64, u64)> = vec![(0, 0); self.size_assets as usize];
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Just return a default AllAssets for testing
    fn create_default_all_assets() -> AllAssets {
        AllAssets {
            base_asset: Pubkey::default(),
            assets: [
                AssetInfo {
                    mint: Pubkey::default(),
                    vault: Pubkey::default(),
                    multiplier: 1,
                    orderbook: Orderbook {
                        slots: [0; ORDERBOOK_SIZE],
                    },
                },
                AssetInfo {
                    mint: Pubkey::default(),
                    vault: Pubkey::default(),
                    multiplier: 1,
                    orderbook: Orderbook {
                        slots: [0; ORDERBOOK_SIZE],
                    },
                },
            ],
            size_assets: 2, // 2 because we have 2 assets listed in this structure
            start_tick: 100,
            tick_size: 10,
            amount: 0,
        }
    }

    #[test]
    fn test_current_best_apy() {
        let mut all_assets = create_default_all_assets();

        // Set up first asset with liquidity at tick index 0 (APY = 100)
        all_assets.assets[0].orderbook.slots[0] = 500;

        // Set up second asset with liquidity at tick index 3 (APY = 130)
        all_assets.assets[1].orderbook.slots[3] = 300;

        let result = all_assets.current_best_apy().unwrap();
        assert_eq!(result, (130, 1, 300)); // Expecting APY=130 from asset index 1 with liquidity 300
    }

    #[test]
    fn test_split_lenders_sol() {
        let mut all_assets = create_default_all_assets();

        all_assets.assets[0].orderbook.slots[0] = 500;
        all_assets.assets[0].orderbook.slots[1] = 400;
        all_assets.assets[0].orderbook.slots[2] = 100;
        // Orderbook for asset0: [500, 400, 100, 0, 0, 0, 0, 0, 0, 0]
        // Aka, 500amount of liquidity at tick 0 (100%), 400 at tick 1 (110%), 100 at tick 2 (120%)

        all_assets.amount = 600;
        let result = all_assets.split_lenders_sol().unwrap();
        // For asset0, we should take 500 from tick 0 and 100 from tick 1
        // so the result for asset0 is (1, 600)
        // For asset1, there is no liquidity, so (0, 0) - but it doesnt matter so if your implem returns smth else when the amount of liquidity is 0 its correct also and just change the test
        assert_eq!(result, vec![(1, 600), (0, 0)]);

        all_assets.amount = 400;
        let result = all_assets.split_lenders_sol().unwrap();
        assert_eq!(result, vec![(1, 400), (0, 0)]);

        // Orderbook for asset1: [100, 100, 200, 300, 0, 0, 0, 0, 0, 0]
        all_assets.assets[1].orderbook.slots[0] = 100;
        all_assets.assets[1].orderbook.slots[1] = 100;
        all_assets.assets[1].orderbook.slots[2] = 200;
        all_assets.assets[1].orderbook.slots[3] = 300;
        // Aka, 100amount of liquidity at tick 0 (100%), 100 at tick 1 (110%), 200 at tick 2 (120%), 300 at tick 3 (130%)

        all_assets.amount = 100;
        let result = all_assets.split_lenders_sol().unwrap();
        // The best APY is now from asset1 at tick 3, so we should take 100 from there
        assert_eq!(result, vec![(0, 0), (3, 100)]);

        all_assets.amount = 600;
        let result = all_assets.split_lenders_sol().unwrap();
        // We should take 300 from tick 3 of asset1, we now have 300 left to split
        // The next best APY is tick 2 of asset1, so we take 200 from there, 100 left
        // The next best APY is tick 2 of asset0, so we take 100 from there, 0 left - done
        assert_eq!(result, vec![(2, 100), (3, 500)]);
    }
}