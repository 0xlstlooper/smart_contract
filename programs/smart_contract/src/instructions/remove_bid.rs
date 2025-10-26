use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface, TransferChecked, transfer_checked},
};

use crate::state::{AllAssets, LooperDeposit, Orderbook, ORDERBOOK_SIZE};
use crate::errors::ErrorCode;

#[derive(Accounts)]
pub struct RemoveBid<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub all_assets: Account<'info, AllAssets>,
    #[account(
        mut,
        seeds = [b"orderbook", mint_asset.key().as_ref()],
        bump,
    )]
    pub orderbook: Account<'info, Orderbook>,
    pub mint_asset: InterfaceAccount<'info, Mint>,

    // Le "ticket" PDA, qui sera fermé après le retrait de l'offre
    #[account(
        mut,
        close = payer,
        seeds = [b"lender_deposit", payer.key().as_ref(), mint_asset.key().as_ref(), &lender_deposit.slot_index.to_le_bytes()],
        bump = lender_deposit.bump,
        constraint = payer.key() == lender_deposit.lender @ ErrorCode::OnlyOriginalLender,
    )]
    pub lender_deposit: Account<'info, LooperDeposit>,

    // Compte de tokens de l'utilisateur pour l'actif
    #[account(
        mut,
        associated_token::mint = mint_asset,
        associated_token::authority = payer,
    )]
    pub destination_account: InterfaceAccount<'info, TokenAccount>,
    
    // Le coffre-fort pour l'actif
    #[account(
        mut,
        associated_token::mint = mint_asset,
        associated_token::authority = vault_authority,
    )]
    pub vault_asset: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: ok
    #[account(
        seeds = [b"vault_authority"],
        bump
    )]
    pub vault_authority: UncheckedAccount<'info>,

    // Programmes
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<RemoveBid>) -> Result<()> {
    // 1. Obtenir les informations du compte de dépôt
    let lender_deposit = &ctx.accounts.lender_deposit;
    let slot_index = lender_deposit.slot_index as usize;
    let amount = lender_deposit.amount;

    // 2. Calculer l'index du slot
    let all_assets = &ctx.accounts.all_assets;
    require!(slot_index < ORDERBOOK_SIZE, ErrorCode::InvalidTickIndex);

    // 3. Mettre à jour l'orderbook
    let orderbook = &mut ctx.accounts.orderbook;
    orderbook.slots[slot_index] = orderbook.slots[slot_index].checked_sub(amount).unwrap();

    // 4. Renvoyer les tokens à l'utilisateur
    let bump = ctx.bumps.vault_authority;
    let signer_seeds = &[&b"vault_authority"[..], &[bump]];
    let signer = &[&signer_seeds[..]];

    let cpi_accounts = TransferChecked {
        from: ctx.accounts.vault_asset.to_account_info(),
        mint: ctx.accounts.mint_asset.to_account_info(),
        to: ctx.accounts.destination_account.to_account_info(),
        authority: ctx.accounts.vault_authority.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
    transfer_checked(cpi_ctx, amount, ctx.accounts.mint_asset.decimals)?;

    // 5. Le compte lender_deposit est fermé automatiquement par Anchor grâce à la contrainte `close = payer`

    Ok(())
}