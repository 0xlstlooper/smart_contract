use crate::constants::*;
use crate::errors::ErrorCode;
use crate::utility::update_multiplier;
use anchor_lang::prelude::*;

pub const MAX_ASSETS: u64 = 2;
pub const ORDERBOOK_SIZE: usize = 10;

/*
The first index is the lowest APY offered - this is start_apy
The last index is the highest APY offered - index i corresponds to start_apy + i * apy_tick
*/
#[account]
#[derive(InitSpace)]
pub struct Orderbook {
    pub slots: [u64; ORDERBOOK_SIZE],
    pub looper_multiplier: [u64; ORDERBOOK_SIZE],
    pub low_position_decay: [u64; ORDERBOOK_SIZE], // When multiple positions shares the same last slot, we split the leverage among them, so we have this decay multiplier to do so
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct AssetInfo {
    pub mint: Pubkey,
    pub leverage: u64, // Related to the LTV of the asset - leverage of all other selectionned positions
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
    pub start_apy: u64, // 100% is VALUE_100_PERCENT_APY
    pub apy_tick: u64,
    // Total amount of SOL deposited by lenders in this market
    pub amount: u64, // Used, but probably some precision errors with the tracking of interest
    pub lender_multiplier: u64, // Start at 1, increases over time to reflect interest accrued for the lenders
    pub last_update_timestamp: i64,
    pub current_apy: u64,
}

/* Invariant of the structure:
todo

*/

impl AllAssets {
    pub fn update_timestamp_and_multiplier(&mut self) -> Result<()> {
        let current_timestamp = Clock::get()?.unix_timestamp;
        let time_elapsed = current_timestamp
            .checked_sub(self.last_update_timestamp)
            .ok_or(ErrorCode::NumErr)?;
        if time_elapsed > 0 {
            self.amount = update_multiplier(self.current_apy, time_elapsed, self.amount)? as u64;
            self.lender_multiplier =
                update_multiplier(self.current_apy, time_elapsed, self.lender_multiplier)? as u64;
            self.last_update_timestamp = current_timestamp;
        }
        // Todo, voir pcq en fait faudrait calculer ce truc avant et apres, pcq le split peut se mettre à jour avec le calcul des taux d'interet - c'est un peu chiant donc on fait ça comme ça pour l'instant mais todo changer
        let current_split = self.split_lenders_sol()?;
        self.update_looper_multiplier(time_elapsed, &current_split)?;
        // Update apy
        self.update_apy(&current_split)?;
        Ok(())
    }

    pub fn update_looper_multiplier(
        &mut self,
        time_elapsed: i64,
        current_split: &Vec<(u64, u64, u64)>,
    ) -> Result<()> {
        for i in 0..self.size_assets as usize {
            let asset = &mut self.assets[i];
            let (_tick_index, amount, last_liquidity) = current_split[i];
            let tick_index = _tick_index as usize;
            if amount > 0 {
                // Update all the ticks above and including tick_index
                for j in (tick_index)..ORDERBOOK_SIZE {
                    let apy = self.start_apy + (j as u64) * self.apy_tick;
                    asset.orderbook.looper_multiplier[j] =
                        update_multiplier(apy, time_elapsed, asset.orderbook.looper_multiplier[j])?
                            as u64;
                }
                // Update the decay for the current tick
                asset.orderbook.low_position_decay[tick_index] = asset.orderbook.low_position_decay
                    [tick_index]
                    .checked_mul(last_liquidity)
                    .ok_or(ErrorCode::NumErr)?
                    .checked_div(asset.orderbook.slots[tick_index])
                    .ok_or(ErrorCode::NumErr)?;
            }
        }
        Ok(())
    }

    // Update self.current_apy based on the current split of lenders' SOL
    pub fn update_apy(&mut self, current_split: &Vec<(u64, u64, u64)>) -> Result<()> {
        let total_liquidity: u64 = current_split
            .iter()
            .map(|&(_tick_index, amount, _last_liquidity)| amount)
            .sum();
        // Do the mean averaged by liquidity
        if total_liquidity > 0 {
            let mut new_apy: u128 = 0;
            for i in 0..self.size_assets as usize {
                let asset = &self.assets[i];
                let (tick_index, amount, _last_liquidity) = current_split[i];
                if amount > 0 {
                    let apy = self.start_apy + (tick_index as u64) * self.apy_tick;
                    new_apy = new_apy
                        .checked_add(
                            (apy as u128)
                                .checked_mul(amount as u128)
                                .ok_or(ErrorCode::NumErr)?,
                        )
                        .ok_or(ErrorCode::NumErr)?;
                }
            }
            self.current_apy = (new_apy
                .checked_div(total_liquidity as u128)
                .ok_or(ErrorCode::NumErr)?) as u64;
        }
        Ok(())
    }

    // Returns (best_tick, index of the related asset, liquidity of this tick of this asset)
    // Maximize only the tick, not the liquidity - for the same tick, it does not matter which asset we choose
    // Useless ?
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
                    let current_apy = self.start_apy + (j as u64) * self.apy_tick;

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
    // Need to return an array of (tick_index, amount, last_liquidity), which represents for each assets
    //   upon which tick we selected their liquidity, what amount we took from it, and how much liquidity was left in this tick after our selection
    // So: the sum all amounts must be equal to the input amount
    // Result is a vector of size all_assets.size_assets
    // @Audrey
    pub fn split_lenders_sol(&self) -> Result<Vec<(u64, u64, u64)>> {
        // Delegate to helper that uses self.amount
        self.split_lenders_sol_for_amount(self.amount)
    }

    /// Helper: compute split for an arbitrary amount, using the current orderbooks.
    fn split_lenders_sol_for_amount(&self, amount_to_split: u64) -> Result<Vec<(u64, u64, u64)>> {
        // result per asset: (last_tick_used_index, total_amount_taken_from_asset, amount_taken_from_last_tick)
        let mut result: Vec<(u64, u64, u64)> = vec![(0u64, 0u64, 0u64); self.size_assets as usize];

        // Create mutable working copy of liquidity per asset per tick
        let mut slots_matrix: Vec<Vec<u64>> = Vec::with_capacity(self.size_assets as usize);
        for i in 0..self.size_assets as usize {
            let mut v: Vec<u64> = Vec::with_capacity(ORDERBOOK_SIZE);
            for j in 0..ORDERBOOK_SIZE {
                v.push(self.assets[i].orderbook.slots[j]);
            }
            slots_matrix.push(v);
        }

        let mut remaining: u64 = amount_to_split;

        // Iterate ticks from highest APY to lowest (higher index => higher APY)
        for tick in (0..ORDERBOOK_SIZE).rev() {
            if remaining == 0 {
                break;
            }
            // For each tick, iterate through assets in order (asset index increasing).
            // This choice matches the idea "for the same tick it does not matter which asset we choose" -
            // we pick deterministic order.
            for asset_index in 0..self.size_assets as usize {
                if remaining == 0 {
                    break;
                }
                let available = slots_matrix[asset_index][tick];
                if available == 0 {
                    continue;
                }
                let take = if available <= remaining {
                    available
                } else {
                    remaining
                };
                // update result
                let (_, total_taken, _) = result[asset_index];
                let new_total = total_taken.checked_add(take).ok_or(ErrorCode::NumErr)?;
                result[asset_index].1 = new_total;
                // last tick used becomes the current tick (we progress from highest to lowest,
                // so this will end up being the lowest-index tick we consumed for that asset)
                result[asset_index].0 = tick as u64;
                // amount taken from that last tick is exactly 'take' (overwritten if later we take from a lower tick)
                result[asset_index].2 = take;
                // reduce available
                slots_matrix[asset_index][tick] =
                    available.checked_sub(take).ok_or(ErrorCode::NumErr)?;
                // reduce remaining
                remaining = remaining.checked_sub(take).ok_or(ErrorCode::NumErr)?;
            }
        }

        // If we couldn't split entire amount (i.e., not enough liquidity), this function still returns what it could take.
        // Tests in this repository expect that sum of taken equals requested amount when liquidity suffices.
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
    pub fn delta_split_lender(&self, delta: u64, sign: bool) -> Result<Vec<(u64, i128)>> {
        // compute old split for current amount
        let old_split = self.split_lenders_sol_for_amount(self.amount)?;
        // compute new amount
        let new_amount = if sign {
            self.amount.checked_add(delta).ok_or(ErrorCode::NumErr)?
        } else {
            self.amount.checked_sub(delta).ok_or(ErrorCode::NumErr)?
        };
        // compute new split for new amount
        let new_split = self.split_lenders_sol_for_amount(new_amount)?;

        let mut result: Vec<(u64, i128)> = vec![(0u64, 0i128); self.size_assets as usize];
        for i in 0..self.size_assets as usize {
            // we return the tick_index coming from the new split (where the new distribution ended for this asset)
            let new_tick = new_split[i].0;
            let old_amount = old_split[i].1 as i128;
            let new_amount_i128 = new_split[i].1 as i128;
            let diff = new_amount_i128
                .checked_sub(old_amount)
                .ok_or(ErrorCode::NumErr)?;
            result[i] = (new_tick, diff);
        }
        Ok(result)
    }

    /* Basically, same as above, but for loopers
    Whats the new repartition once we apply a change of `delta` to asset `asset_index` at slot `slot_index`
    */
    // @Audrey
    pub fn delta_split_looper(
        &self,
        asset_index: usize,
        slot_index: usize,
        delta: u64,
        sign: bool,
    ) -> Result<Vec<(u64, i128)>> {
        // Validate indices
        if asset_index >= self.size_assets as usize {
            return Err(error!(ErrorCode::NumErr));
        }
        if slot_index >= ORDERBOOK_SIZE {
            return Err(error!(ErrorCode::NumErr));
        }

        // Build slots matrix from current orderbooks
        let mut slots_matrix: Vec<Vec<u64>> = Vec::with_capacity(self.size_assets as usize);
        for i in 0..self.size_assets as usize {
            let mut v: Vec<u64> = Vec::with_capacity(ORDERBOOK_SIZE);
            for j in 0..ORDERBOOK_SIZE {
                v.push(self.assets[i].orderbook.slots[j]);
            }
            slots_matrix.push(v);
        }

        // Apply change to the specific asset/slot
        if sign {
            // add
            slots_matrix[asset_index][slot_index] = slots_matrix[asset_index][slot_index]
                .checked_add(delta)
                .ok_or(ErrorCode::NumErr)?;
        } else {
            // remove - ensure we don't underflow
            if slots_matrix[asset_index][slot_index] < delta {
                // If trying to remove more than available, cap at zero (alternative: return error).
                slots_matrix[asset_index][slot_index] = 0;
            } else {
                slots_matrix[asset_index][slot_index] = slots_matrix[asset_index][slot_index]
                    .checked_sub(delta)
                    .ok_or(ErrorCode::NumErr)?;
            }
        }

        // Helper to compute split from arbitrary slots matrix (same semantics as split_lenders_sol_for_amount)
        fn compute_split_from_slots(
            amount_to_split: u64,
            slots_matrix: &Vec<Vec<u64>>,
            size_assets: usize,
        ) -> Result<Vec<(u64, u64, u64)>> {
            let mut result: Vec<(u64, u64, u64)> = vec![(0u64, 0u64, 0u64); size_assets];
            let mut remaining = amount_to_split;
            let mut local_slots = slots_matrix.clone();

            for tick in (0..ORDERBOOK_SIZE).rev() {
                if remaining == 0 {
                    break;
                }
                for asset_idx in 0..size_assets {
                    if remaining == 0 {
                        break;
                    }
                    let available = local_slots[asset_idx][tick];
                    if available == 0 {
                        continue;
                    }
                    let take = if available <= remaining {
                        available
                    } else {
                        remaining
                    };
                    let (_, total_taken, _) = result[asset_idx];
                    let new_total = total_taken.checked_add(take).ok_or(ErrorCode::NumErr)?;
                    result[asset_idx].1 = new_total;
                    result[asset_idx].0 = tick as u64;
                    result[asset_idx].2 = take;
                    local_slots[asset_idx][tick] =
                        available.checked_sub(take).ok_or(ErrorCode::NumErr)?;
                    remaining = remaining.checked_sub(take).ok_or(ErrorCode::NumErr)?;
                }
            }
            Ok(result)
        }

        // Compute old split (current state)
        let old_split = compute_split_from_slots(
            self.amount,
            &{
                // build current slots matrix again for immutability
                let mut s: Vec<Vec<u64>> = Vec::with_capacity(self.size_assets as usize);
                for i in 0..self.size_assets as usize {
                    let mut v: Vec<u64> = Vec::with_capacity(ORDERBOOK_SIZE);
                    for j in 0..ORDERBOOK_SIZE {
                        v.push(self.assets[i].orderbook.slots[j]);
                    }
                    s.push(v);
                }
                s
            },
            self.size_assets as usize,
        )?;

        // Compute new split with modified slots
        let new_split =
            compute_split_from_slots(self.amount, &slots_matrix, self.size_assets as usize)?;

        // Return deltas: (tick_index_from_new_split, signed_difference_in_amount)
        let mut result: Vec<(u64, i128)> = vec![(0u64, 0i128); self.size_assets as usize];
        for i in 0..self.size_assets as usize {
            let new_tick = new_split[i].0;
            let old_amt = old_split[i].1 as i128;
            let new_amt = new_split[i].1 as i128;
            let diff = new_amt.checked_sub(old_amt).ok_or(ErrorCode::NumErr)?;
            result[i] = (new_tick, diff);
        }

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
                    leverage: 1,
                    orderbook: Orderbook {
                        slots: [0; ORDERBOOK_SIZE],
                        looper_multiplier: [START_MULTIPLIER_VALUE; ORDERBOOK_SIZE],
                        low_position_decay: [START_DECAY_VALUE; ORDERBOOK_SIZE],
                    },
                },
                AssetInfo {
                    mint: Pubkey::default(),
                    leverage: 1,
                    orderbook: Orderbook {
                        slots: [0; ORDERBOOK_SIZE],
                        looper_multiplier: [START_MULTIPLIER_VALUE; ORDERBOOK_SIZE],
                        low_position_decay: [START_DECAY_VALUE; ORDERBOOK_SIZE],
                    },
                },
            ],
            size_assets: 2, // 2 because we have 2 assets listed in this structure
            start_apy: VALUE_100_PERCENT_APY,
            apy_tick: VALUE_100_PERCENT_APY / 100,
            amount: 0,
            lender_multiplier: START_MULTIPLIER_VALUE,
            last_update_timestamp: 0,
            current_apy: VALUE_100_PERCENT_APY,
        }
    }

    #[test]
    fn test_split_lenders_sol() {
        let mut all_assets = create_default_all_assets();

        all_assets.assets[0].orderbook.slots[0] = 500;
        all_assets.assets[0].orderbook.slots[1] = 400;
        all_assets.assets[0].orderbook.slots[2] = 100;
        // Orderbook for asset0: [500, 400, 100, 0, 0, 0, 0, 0, 0, 0]
        // Aka, 500amount of liquidity at tick 0 (100%), 400 at tick 1 (110%), 100 at tick 2 (120%)

        all_assets.amount = 500;
        let result = all_assets.split_lenders_sol().unwrap();
        // For asset0, we should take 100 from tick 2, and 400 from tick 1
        // so the result for asset0 is (1, 500, 400)
        // For asset1, there is no liquidity, so (0, 0, 0) - but it doesnt matter so if your implem returns smth else when the amount of liquidity is 0 its correct also and just change the test
        assert_eq!(result, vec![(1, 500, 400), (0, 0, 0)]);

        all_assets.amount = 300;
        let result = all_assets.split_lenders_sol().unwrap();
        assert_eq!(result, vec![(1, 300, 200), (0, 0, 0)]);

        all_assets.amount = 100;
        let result = all_assets.split_lenders_sol().unwrap();
        assert_eq!(result, vec![(2, 100, 100), (0, 0, 0)]);

        // Orderbook for asset1: [100, 100, 200, 300, 0, 0, 0, 0, 0, 0]
        all_assets.assets[1].orderbook.slots[0] = 100;
        all_assets.assets[1].orderbook.slots[1] = 100;
        all_assets.assets[1].orderbook.slots[2] = 200;
        all_assets.assets[1].orderbook.slots[3] = 300;
        // Aka, 100amount of liquidity at tick 0 (100%), 100 at tick 1 (110%), 200 at tick 2 (120%), 300 at tick 3 (130%)

        all_assets.amount = 100;
        let result = all_assets.split_lenders_sol().unwrap();
        // The best APY is now from asset1 at tick 3, so we should take 100 from there
        assert_eq!(result, vec![(0, 0, 0), (3, 100, 100)]);

        all_assets.amount = 600;
        let result = all_assets.split_lenders_sol().unwrap();
        // We should take 300 from tick 3 of asset1, we now have 300 left to split
        // The next best APY is tick 2 of asset1, so we take 200 from there, 100 left
        // The next best APY is tick 2 of asset0, so we take 100 from there, 0 left - done
        assert_eq!(result, vec![(2, 100, 100), (2, 500, 200)]);
    }
}
