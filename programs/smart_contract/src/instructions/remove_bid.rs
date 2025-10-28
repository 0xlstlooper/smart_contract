use crate::errors::ErrorCode;
use crate::state::{AllAssets, LooperDeposit, Orderbook, ORDERBOOK_SIZE};
use crate::utility::*;
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

#[derive(Accounts)]
#[instruction(asset_index: u64, slot_index: u64)]
pub struct RemoveBid<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"all_assets", all_assets.base_asset.key().as_ref()],
        bump,
    )]
    pub all_assets: Account<'info, AllAssets>,

    // Looper deposit to be closed
    #[account(
        mut,
        close = payer,
        seeds = [b"looper_deposit", payer.key().as_ref(), &asset_index.to_le_bytes(), &slot_index.to_le_bytes()],
        bump = looper_deposit.bump,
    )]
    pub looper_deposit: Account<'info, LooperDeposit>,

    /// CHECK: ok
    #[account(
        seeds = [b"vault_authority"],
        bump
    )]
    pub vault_authority: UncheckedAccount<'info>,

    // Programs
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler<'info>(
    ctx: Context<'_, '_, 'info, 'info, RemoveBid<'info>>,
    asset_index: usize,
    slot_index: usize,
) -> Result<()> {
    // Update global multiplier
    ctx.accounts.all_assets.update_timestamp_and_multiplier()?;
    ctx.accounts.looper_deposit.adjust_for_looper_multiplier(
        ctx.accounts.all_assets.assets[asset_index]
            .orderbook
            .looper_multiplier[slot_index] as u128,
    )?;
    ctx.accounts.looper_deposit.adjust_for_decay(
        ctx.accounts.all_assets.assets[asset_index]
            .orderbook
            .low_position_decay[slot_index] as u128,
    )?;

    let amount = ctx.accounts.looper_deposit.amount;
    require!(
        asset_index < ctx.accounts.all_assets.size_assets as usize,
        ErrorCode::InvalidAssetIndex
    );
    require!(slot_index < ORDERBOOK_SIZE, ErrorCode::InvalidSlotIndex);

    // Update the book
    let all_assets = &mut ctx.accounts.all_assets;
    all_assets.assets[asset_index].orderbook.slots[slot_index] =
        all_assets.assets[asset_index].orderbook.slots[slot_index]
            .checked_sub(amount)
            .ok_or(ErrorCode::NumErr)?;

    // Delta splits for withdrawal. `false` indicates a withdrawal.
    let delta_split = all_assets.delta_split_looper(asset_index, slot_index, amount, false)?;

    // For a simple withdrawal, we expect the deposit part to be empty
    let ((deposit_amounts, _), (withdraw_amounts, withdraw_mints)) =
        delta_split_extraction(&delta_split, &ctx.accounts.all_assets);

    // Ensure no deposits are being made - is it actually the case, todo verifier
    require!(
        deposit_amounts.is_empty(),
        ErrorCode::ShouldBeNoDepositAmountsInRemoveBid
    );

    // Prepare the signer seeds for the vault authority PDA
    let bump = ctx.bumps.vault_authority;
    let signer_seeds = &[&b"vault_authority"[..], &[bump]];
    let signer = &[&signer_seeds[..]];

    // Withdraw the assets back to the user using the same helper function
    manage_withdraw(
        &withdraw_amounts,
        &withdraw_mints,
        &ctx.remaining_accounts, // All remaining accounts are for withdrawals
        &ctx.accounts.payer,
        &ctx.accounts.vault_authority,
        &mut ctx.accounts.token_program,
        signer,
    )?;

    // The looper_deposit account is closed automatically by Anchor via the `close = payer` constraint.

    Ok(())
}
