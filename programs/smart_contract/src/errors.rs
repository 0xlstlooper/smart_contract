use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Asset already initialized")]
    AssetAlreadyInitialized,
    #[msg("The struct AllAsssets is full")]
    AllAsssetsIsFull,
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