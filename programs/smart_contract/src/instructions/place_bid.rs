use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface, TransferChecked, transfer_checked},
};

use crate::state::{AllAssets, LenderDeposit, Orderbook, ORDERBOOK_SIZE};
use crate::errors::ErrorCode;

#[derive(Accounts)]
#[instruction(tick: u64)]
pub struct PlaceBid<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub allassets: Account<'info, AllAssets>,
    #[account(
        mut,
        seeds = [b"orderbook", mint_asset.key().as_ref()],
        bump,
    )]
    pub orderbook: Account<'info, Orderbook>,
    pub mint_asset: InterfaceAccount<'info, Mint>,

    // Ce PDA sert de "ticket" pour que le prêteur puisse retirer son offre plus tard.
    #[account(
        init,
        payer = payer,
        space = 8 + LenderDeposit::INIT_SPACE,
        seeds = [b"lender_deposit", payer.key().as_ref(), mint_asset.key().as_ref(), &tick.to_le_bytes()],
        bump,
    )]
    pub lender_deposit: Account<'info, LenderDeposit>,

    // Compte de tokens de l'utilisateur pour l'actif
    #[account(
        mut,
        associated_token::mint = mint_asset,
        associated_token::authority = payer,
    )]
    pub source_account: InterfaceAccount<'info, TokenAccount>,
    
    // Le coffre-fort (vault) pour l'actif
    #[account(
        mut,
        associated_token::mint = mint_asset,
        associated_token::authority = vault_authority,
    )]
    pub vault_asset: InterfaceAccount<'info, TokenAccount>,

    // Autorité du coffre-fort
    #[account(
        seeds = [b"vault_authority"],
        bump
    )]
    /// CHECK: C'est juste un PDA, pas un compte de programme.
    pub vault_authority: AccountInfo<'info>,

    // Programmes
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<PlaceBid>, slot_index: usize, amount: u64) -> Result<()> {
    // 1. Calculer l'index du slot à partir du tick
    let allassets = &ctx.accounts.allassets;
    let orderbook = &mut ctx.accounts.orderbook;

    require!(slot_index < ORDERBOOK_SIZE, ErrorCode::InvalidTickIndex);
    // require!(tick >= allassets.start_tick, ErrorCode::TickTooLow);
    // require!(tick <= allassets.start_tick + allassets.tick_size * (ORDERBOOK_SIZE as u64 - 1), ErrorCode::TickTooHigh);
    // require!(tick % allassets.tick_size == 0, ErrorCode::TickNotAligned);
    // let slot_index = ((tick - allassets.start_tick) / allassets.tick_size) as usize;
    // require!(slot_index < ORDERBOOK_SIZE, ErrorCode::InvalidTickIndex); // Shouldn't happen

    // 2. Mettre à jour l'orderbook
    orderbook.slots[slot_index] = orderbook.slots[slot_index].checked_add(amount).unwrap();

    // 3. Transférer les tokens vers le coffre-fort
    let cpi_accounts = TransferChecked {
        from: ctx.accounts.source_account.to_account_info(),
        mint: ctx.accounts.mint_asset.to_account_info(),
        to: ctx.accounts.vault_asset.to_account_info(),
        authority: ctx.accounts.payer.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    transfer_checked(cpi_ctx, amount, ctx.accounts.mint_asset.decimals)?;

    // 4. Initialiser le compte de dépôt du prêteur (le ticket)
    let lender_deposit = &mut ctx.accounts.lender_deposit;
    lender_deposit.lender = ctx.accounts.payer.key();
    lender_deposit.mint_asset = ctx.accounts.mint_asset.key();
    lender_deposit.slot_index = slot_index as u64;
    lender_deposit.amount = amount;
    lender_deposit.bump = ctx.bumps.lender_deposit;

    Ok(())
}