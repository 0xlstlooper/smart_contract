use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    // Deposit
    #[msg("The number of amounts does not match the number of asset account sets.")]
    InvalidInputLength,
    #[msg("Account mints do not match.")]
    MintMismatch,
    #[msg("Source account owner is not the payer.")]
    OwnerMismatch,
    #[msg("Vault asset account owner is not the vault authority.")]
    VaultOwnerMismatch,
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
}