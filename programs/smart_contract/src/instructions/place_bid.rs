use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface, TransferChecked, transfer_checked},
};
use crate::errors::ErrorCode;
use crate::state::{AllAssets, LooperDeposit, Orderbook, ORDERBOOK_SIZE};
use crate::utility::*;
use crate::constants::*;

#[derive(Accounts)]
#[instruction(asset_index: u64, slot_index: u64)]
pub struct PlaceBid<'info> {

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"all_assets", all_assets.base_asset.key().as_ref()],
        bump,
    )]
    pub all_assets: Account<'info, AllAssets>,

    #[account(
        init, // Todo, init if needed
        payer = payer,
        space = 8 + LooperDeposit::INIT_SPACE,
        seeds = [b"looper_deposit", payer.key().as_ref(), &asset_index.to_le_bytes(), &slot_index.to_le_bytes()],
        bump,
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
    ctx: Context<'_, '_, 'info, 'info, PlaceBid<'info>>,
    asset_index: usize,
    slot_index: usize,
    amount: u64,
) -> Result<()> {

    // Update global multiplier - todo faudrait factoriser cette merde qq part
    ctx.accounts.all_assets.update_timestamp_and_multiplier()?;

    require!(asset_index < ctx.accounts.all_assets.size_assets as usize, ErrorCode::InvalidAssetIndex);
    require!(slot_index < ORDERBOOK_SIZE, ErrorCode::InvalidSlotIndex);
    require!(amount > MIN_DEPOSIT, ErrorCode::LuserEstUnRat);

    // Update the book
    let all_assets = &mut ctx.accounts.all_assets;
    all_assets.assets[asset_index].orderbook.slots[slot_index] = all_assets.assets[asset_index].orderbook.slots[slot_index].checked_add(amount).ok_or(ErrorCode::NumErr)?;

    // Delta splits
    let delta_split = all_assets.delta_split_looper(asset_index, slot_index, amount, true)?;
    let ((deposit_amounts, deposit_mints), (withdraw_amounts, withdraw_mints)) = delta_split_extraction(&delta_split, &ctx.accounts.all_assets);

    // Split the remaining accounts accordingly between deposit and withdraw
    let deposit_remaining_accounts = &ctx.remaining_accounts[0..3*deposit_amounts.len()];
    let withdraw_remaining_accounts = &ctx.remaining_accounts[3*deposit_amounts.len()..];

    // Deposit
    manage_deposit(
        &deposit_amounts,
        &deposit_mints,
        &deposit_remaining_accounts,
        &ctx.accounts.payer,
        &ctx.accounts.vault_authority,
        &mut ctx.accounts.token_program,
    )?;

    // Prepare the signer seeds for the vault authority PDA
    let bump = ctx.bumps.vault_authority;
    let signer_seeds = &[&b"vault_authority"[..], &[bump]];
    let signer = &[&signer_seeds[..]];

    // Withdraw
    manage_withdraw(
        &withdraw_amounts,
        &withdraw_mints,
        &withdraw_remaining_accounts,
        &ctx.accounts.payer,
        &ctx.accounts.vault_authority,
        &mut ctx.accounts.token_program,
        signer,
    )?;

    // Initialize looper_deposit account
    let looper_deposit = &mut ctx.accounts.looper_deposit;
    looper_deposit.looper = ctx.accounts.payer.key();
    looper_deposit.asset_index = asset_index as u64;
    looper_deposit.slot_index = slot_index as u64;
    looper_deposit.amount = amount;
    looper_deposit.last_multiplier = ctx.accounts.all_assets.assets[asset_index].orderbook.looper_multiplier[slot_index];
    looper_deposit.last_decay = ctx.accounts.all_assets.assets[asset_index].orderbook.low_position_decay[slot_index];
    looper_deposit.bump = ctx.bumps.looper_deposit;

    Ok(())
}