use anchor_lang::{prelude::*, system_program};
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface, TransferChecked, transfer_checked},
};
use crate::errors::ErrorCode;
use crate::state::{AllAssets, LenderDeposit};
use crate::utility::*;

// Todo check the re init attack

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct Deposit<'info> {

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"all_assets", all_assets.base_asset.key().as_ref()],
        bump,
    )]
    pub all_assets: Account<'info, AllAssets>,

    #[account(
        init_if_needed,
        payer = payer,
        space = 8 + LenderDeposit::INIT_SPACE,
        seeds = [b"lender_deposit", payer.key().as_ref()],
        bump,
    )]
    pub lender_deposit: Account<'info, LenderDeposit>,

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

// Structure of remaining_accounts: for each asset being deposited: [source_account, vault_asset, mint_asset]
pub fn handler<'info>(
    ctx: Context<'_, '_, 'info, 'info, Deposit<'info>>, 
    amount: u64
) -> Result<()> {

    require!(amount > 0, ErrorCode::LuserEstUnRat);

    // Update global multiplier
    ctx.accounts.all_assets.update_timestamp_and_multiplier()?;
    ctx.accounts.all_assets.amount = ctx.accounts.all_assets.amount.checked_add(amount).ok_or(ErrorCode::NumErr)?;

    // The amounts we are supposed to deposit for each asset. `true` for deposit.
    let delta_split = ctx.accounts.all_assets.delta_split_lender(amount, true)?;
    let ((deposit_amounts, deposit_mints), (withdraw_amounts, withdraw_mints)) = delta_split_extraction(&delta_split, &ctx.accounts.all_assets);
    require!(withdraw_amounts.len() == 0, ErrorCode::ShouldBeNoWithdrawAmounts);

    // Do the actual deposits
    manage_deposit(
        &deposit_amounts,
        &deposit_mints,
        &ctx.remaining_accounts,
        &ctx.accounts.payer,
        &ctx.accounts.vault_authority,
        &mut ctx.accounts.token_program,
    )?;

    // Update lender_deposit account
    let lender_deposit = &mut ctx.accounts.lender_deposit;
    lender_deposit.lender = ctx.accounts.payer.key();
    // First initialization we just set the multiplier and amount
    if lender_deposit.last_multiplier == 0 {
        lender_deposit.amount = amount;
        lender_deposit.last_multiplier = ctx.accounts.all_assets.lender_multiplier;
    } else {
        lender_deposit.adjust_for_lender_multiplier(ctx.accounts.all_assets.lender_multiplier as u128)?;
        lender_deposit.amount = lender_deposit.amount.checked_add(amount).ok_or(ErrorCode::NumErr)?;
    }
    lender_deposit.bump = ctx.bumps.lender_deposit;

    Ok(())
}