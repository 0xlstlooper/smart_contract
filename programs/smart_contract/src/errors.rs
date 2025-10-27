use anchor_lang::prelude::*;

// Todo group these errors codes together

#[error_code]
pub enum ErrorCode {
    // Deposit
    #[msg("Ser deposit at least something 🐀")]
    LuserEstUnRat,
    #[msg("The number of amounts does not match the number of asset account sets.")]
    InvalidInputLength,
    #[msg("Account mints do not match.")]
    MintMismatch,
    #[msg("Source account owner is not the payer.")]
    OwnerMismatch,
    #[msg("Vault asset account owner is not the vault authority.")]
    VaultOwnerMismatch,
    // Withdraw
    #[msg("Insufficient funds in the vault for withdrawal.")]
    InsufficientVaultFunds,
    // Add Asset
    #[msg("Asset already initialized")]
    AssetAlreadyInitialized,
    #[msg("The struct AllAssets is full")]
    AllAssetsIsFull,
    #[msg("No liquidity available in the orderbook.")]
    NoLiquidityAvailable,
    #[msg("Tick is below the allowed range.")]
    TickTooLow,
    #[msg("Tick is above the allowed range.")]
    TickTooHigh,
    #[msg("Tick should be aligned with tick size.")]
    TickNotAligned,
    #[msg("Invalid tick index.")]
    InvalidTickIndex,
    #[msg("Only the original lender can remove their bid.")]
    OnlyOriginalLender,
    // Place bid
    #[msg("Asset index must be below size_assets.")]
    InvalidAssetIndex,
    #[msg("Slot index must be below ORDERBOOK_SIZE.")]
    InvalidSlotIndex,
    // Break of invariant of the data structures
    #[msg("Data structure invariant broken: there should be no deposit amounts in a withdraw request.")]
    ShouldBeNoDepositAmounts,
    #[msg("Data structure invariant broken: there should be no withdraw amounts in a deposit request.")]
    ShouldBeNoWithdrawAmounts,
    #[msg("Data structure invariant broken: there should be no deposit amounts in a remove bid request.")]
    ShouldBeNoDepositAmountsInRemoveBid,
    // Number overflow/underflow
    #[msg("Number overflow/underflow occurred.")]
    NumErr,
}