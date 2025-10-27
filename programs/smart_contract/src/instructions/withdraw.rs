use anchor_lang::{prelude::*, system_program};
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};
use crate::errors::ErrorCode;
use crate::state::{AllAssets, LenderDeposit};
use crate::manage_transfer::*;

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"all_assets", all_assets.base_asset.key().as_ref()],
        bump,
    )]
    pub all_assets: Account<'info, AllAssets>,

    #[account(
        mut,
        seeds = [b"lender_deposit", payer.key().as_ref()],
        bump,
        constraint = payer.key() == lender_deposit.lender @ ErrorCode::OnlyOriginalLender,
    )]
    pub lender_deposit: Account<'info, LenderDeposit>,

    /// CHECK: ok
    #[account(
        seeds = [b"vault_authority"],
        bump
    )]
    pub vault_authority: UncheckedAccount<'info>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

// Structure of remaining_accounts: for each asset being withdrawn: [destination_account, vault_asset, mint_asset]
pub fn handler<'info>(
    ctx: Context<'_, '_, 'info, 'info, Withdraw<'info>>,
    amount: u64
) -> Result<()> {

    // Update global multiplier - before deposit
    ctx.accounts.all_assets.update_timestamp_and_multiplier()?;
    ctx.accounts.all_assets.amount = ctx.accounts.all_assets.amount.checked_sub(amount).ok_or(ErrorCode::NumErr)?;

    // Update lender_deposit account
    let lender_deposit = &mut ctx.accounts.lender_deposit;
    lender_deposit.adjust_for_global_multiplier(ctx.accounts.all_assets.global_multiplier as u128)?;
    
    // Now, the total money we can withdraw at max is lender_deposit.amount
    // To ease the user, if amount > lender_deposit.amount, we just withdraw all the money
    // - this is tricky for the frontend because well if amount changes, the delta_split changes too, so are the tokens we get and the swap we need to execute, but yeah
    let amount = if amount > lender_deposit.amount {
        lender_deposit.amount
    } else {
        amount
    };
    // Now, we proceed to the rest of the code
    lender_deposit.amount = lender_deposit.amount.checked_sub(amount).ok_or(ErrorCode::InsufficientVaultFunds)?;

    // The amounts we are supposed to withdraw for each asset. `false` for withdrawal.
    let delta_split = ctx.accounts.all_assets.delta_split_lender(amount, false)?;
    let ((deposit_amounts, deposit_mints), (withdraw_amounts, withdraw_mints)) = delta_split_extraction(&delta_split, &ctx.accounts.all_assets);
    require!(deposit_amounts.len() == 0, ErrorCode::ShouldBeNoDepositAmounts);
    
    // Prepare the signer seeds for the vault authority PDA
    let bump = ctx.bumps.vault_authority;
    let signer_seeds = &[&b"vault_authority"[..], &[bump]];
    let signer = &[&signer_seeds[..]];

    manage_withdraw(
        &withdraw_amounts,
        &withdraw_mints,
        &ctx.remaining_accounts,
        &ctx.accounts.payer,
        &ctx.accounts.vault_authority,
        &mut ctx.accounts.token_program,
        signer,
    )?;

    // Close the lender_deposit account if amount is zero
    if lender_deposit.amount == 0 {
        lender_deposit.close(ctx.accounts.payer.to_account_info())?;
    }

    Ok(())
}