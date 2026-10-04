/*
 * Copyright (c) 2026 PayFreight. All rights reserved.
 * Author: PayFreight Protocol Team
 * 
 * This source code is proprietary and confidential. 
 * Unauthorized copying of this file, via any medium, is strictly prohibited.
*/

/*
 * ============================================================================
 * PAYFREIGHT PROTOCOL - SOLANA SETTLEMENT PROGRAM (ANCHOR)
 * ============================================================================
 * 
 * ARCHITECTURE OVERVIEW:
 * 1. Invisible Web3 / Invisible Escrow: Lock funds in EURC (stablecoin) upon 
 *    freight creation.
 * 2. Automated Settlement: Instant release of funds to the carrier/payout vault 
 *    upon Proof of Delivery (e-PoD) verification.
 * 3. Token Utility & Dynamic Fee Discount: 
 *    - Standard protocol fee (neither holds tokens) = 1.5% (150 bps)
 *    - Single party holds utility tokens = 1.0% (100 bps)
 *    - Both parties hold utility tokens = 0.5% (50 bps)
 * 
 * Target Environment: Solana Mainnet / Devnet
 * Framework: Anchor
 * ============================================================================
 */

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

declare_id!("PayFreightProtocol1111111111111111111111111111");

#[program]
pub mod payfreight_settlement {
    use super::*;

    /// 1. Shipper locks EURC stablecoin into the freight escrow account
    pub fn create_freight_escrow(
        ctx: Context<CreateEscrow>,
        freight_id: String,
        amount_eurc: u64,
    ) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow_account;
        escrow.shipper = ctx.accounts.shipper.key();
        escrow.carrier = ctx.accounts.carrier.key();
        escrow.amount = amount_eurc;
        escrow.freight_id = freight_id;
        escrow.is_settled = false;

        // Transfer EURC from shipper wallet to the escrow vault
        let cpi_accounts = Transfer {
            from: ctx.accounts.shipper_eurc.to_account_info(),
            to: ctx.accounts.escrow_vault.to_account_info(),
            authority: ctx.accounts.shipper.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        token::transfer(CpiContext::new(cpi_program, cpi_accounts), amount_eurc)?;

        Ok(())
    }

    /// 2. Release payment upon e-PoD verification with multi-tier dynamic fee discount
    pub fn release_payment_with_discount(
        ctx: Context<ReleasePayment>,
        shipper_has_tokens: bool,
        carrier_has_tokens: bool,
    ) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow_account;
        require!(!escrow.is_settled, CustomError::AlreadySettled);

        // Calculate dynamic protocol fee based on token holdings:
        // - Both hold tokens: 0.5% (50 bps)
        // - One holds tokens: 1.0% (100 bps)
        // - Neither holds tokens: 1.5% (150 bps)
        let fee_bps: u64 = match (shipper_has_tokens, carrier_has_tokens) {
            (true, true) => 50,
            (true, false) | (false, true) => 100,
            (false, false) => 150,
        };
        
        let protocol_fee = (escrow.amount * fee_bps) / 10000;
        let carrier_payout = escrow.amount - protocol_fee;

        // Perform payout transfer to carrier vault / off-ramp handler
        let cpi_accounts = Transfer {
            from: ctx.accounts.escrow_vault.to_account_info(),
            to: ctx.accounts.carrier_eurc.to_account_info(),
            authority: ctx.accounts.escrow_authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        token::transfer(CpiContext::new(cpi_program, cpi_accounts), carrier_payout)?;

        escrow.is_settled = true;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreateEscrow<'info> {
    #[account(init, payer = shipper, space = 8 + 32 + 32 + 8 + 64 + 1)]
    pub escrow_account: Account<'info, FreightEscrow>,
    #[account(mut)]
    pub shipper: Signer<'info>,
    pub carrier: SystemAccount<'info>,
    #[account(mut)]
    pub shipper_eurc: Account<'info, TokenAccount>,
    #[account(mut)]
    pub escrow_vault: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ReleasePayment<'info> {
    #[account(mut)]
    pub escrow_account: Account<'info, FreightEscrow>,
    pub escrow_authority: Signer<'info>,
    #[account(mut)]
    pub escrow_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub carrier_eurc: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct FreightEscrow {
    pub shipper: Pubkey,
    pub carrier: Pubkey,
    pub amount: u64,
    pub freight_id: String,
    pub is_settled: bool,
}

#[error_code]
pub enum CustomError {
    #[msg("This freight payment has already been settled.")]
    AlreadySettled,
}
