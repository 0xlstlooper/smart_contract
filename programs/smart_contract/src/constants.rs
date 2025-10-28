
// Scale of the global multiplier
pub const SCALE_APY: u64 = 1_000;

// Scale of the global multiplier
pub const START_MULTIPLIER_VALUE: u64 = 1_000_000_000_000;

// Start of the decay value
pub const START_DECAY_VALUE: u64 = 1_000_000_000_000_000;

// Scale of the leverage
pub const SCALE_LEVERAGE: u64 = 1_000;

// Scale of the leverage
pub const SCALE_ORACLE_VALUE: u64 = 1_000_000_000_000;

// Safety margin for liquidations
pub const LIQUIDATION_MARGIN: u64 = 1_000; // Accounts that hold below this value will be liquidated
pub const MIN_DEPOSIT: u64 = 0_500 * LIQUIDATION_MARGIN;