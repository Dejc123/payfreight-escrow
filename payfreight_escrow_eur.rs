/*
 * Copyright (c) 2026 PayFreight. All rights reserved.
 * Author: PayFreight Protocol Team
 * 
 * =========================================================================================
 * E-CMR COMPLIANCE, AUTOMATED SETTLEMENT & DISPUTE ARBITRATION MODULE
 * =========================================================================================
 * 
 * LEGAL FRAMEWORK & REGULATORY COMPLIANCE:
 * 1. UNECE Additional Protocol to the CMR Convention concerning the Electronic Consignment 
 *    Note (e-CMR) - Articles 5 (Authenticity), 6 (Contract Details), 8 & 9 (Delivery/Reservations).
 * 2. EU Regulation (EU) 2020/1056 on Electronic Freight Transport Information (eFTI).
 * 3. CMR Convention Article 30 (Reservations & Claims for Loss/Damage).
 * 
 * DISPUTE RESOLUTION & ARBITRATION ARCHITECTURE:
 * Standard flow is 100% automated via Consignee sign-off. However, to prevent deadlocks:
 * - TIMEOUT RELEASE: If Consignee fails to respond within `DISPUTE_WINDOW_SECONDS`, 
 *   Carrier can claim funds automatically (presumption of delivery without reservations).
 * - DISPUTE INITIATION: Shipper or Consignee can initiate a Dispute prior to settlement.
 * - ARBITRATION: Designated Arbitrator (Dispute Board/Platform) can resolve claims 
 *   by split-disbursement based on judicial or independent surveyor reports.
 */

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

declare_id!("PayfrE2222222222222222222222222222222222222");

pub const DISPUTE_WINDOW_SECONDS: i64 = 14 * 24 * 60 * 60; // 14-dnevni rok za reklamacije

#[program]
pub mod payfreight_escrow_eur {
    use super::*;

    /// 1. Initializes the Escrow, registers Shipper, Carrier, Consignee, and Arbitrator.
    pub fn initialize_escrow_eur(
        ctx: Context<InitializeEscrowEur>,
        order_id: String,
        eurc_amount: u64,
        payfreight_amount: u64,
    ) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow_account;
        escrow.shipper = ctx.accounts.shipper.key();
        escrow.carrier = ctx.accounts.carrier.key();
        escrow.consignee = ctx.accounts.consignee.key();
        escrow.arbitrator = ctx.accounts.arbitrator.key(); // Pravno imenovani razsodnik/arbitar
        escrow.eurc_amount = eurc_amount;
        escrow.payfreight_amount = payfreight_amount;
        escrow.order_id = order_id;
        escrow.cmr_hash = String::from("");
        escrow.is_cmr_uploaded = false;
        escrow.cmr_uploaded_at = 0;
        escrow.is_completed = false;
        escrow.is_disputed = false;
        escrow.bump = ctx.bumps.escrow_account;

        // Transfer EURC to Vault
        let cpi_eurc_accounts = Transfer {
            from: ctx.accounts.shipper_eurc_account.to_account_info(),
            to: ctx.accounts.eurc_vault.to_account_info(),
            authority: ctx.accounts.shipper.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        token::transfer(CpiContext::new(cpi_program.clone(), cpi_eurc_accounts), eurc_amount)?;

        // Transfer PAYFREIGHT Collateral to Vault
        let cpi_payfreight_accounts = Transfer {
            from: ctx.accounts.shipper_payfreight_account.to_account_info(),
            to: ctx.accounts.payfreight_vault.to_account_info(),
            authority: ctx.accounts.shipper.to_account_info(),
        };
        token::transfer(CpiContext::new(cpi_program, cpi_payfreight_accounts), payfreight_amount)?;

        msg!("EUR Escrow initialized with Arbitration fallback enabled.");
        Ok(())
    }

    /// 2. Carrier uploads the SHA-256 hash of the signed e-CMR document.
    pub fn upload_cmr_eur(ctx: Context<UploadCMREur>, cmr_hash: String) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow_account;

        require!(!escrow.is_completed, EscrowError::AlreadyCompleted);
        require!(!escrow.is_disputed, EscrowError::EscrowInDispute);
        require!(ctx.accounts.carrier.key() == escrow.carrier, EscrowError::UnauthorizedCarrier);

        let clock = Clock::get()?;
        escrow.cmr_hash = cmr_hash;
        escrow.is_cmr_uploaded = true;
        escrow.cmr_uploaded_at = clock.unix_timestamp; // Zabeležimo čas dostave/nalaganja

        msg!("e-CMR hash anchored on-chain. Timeout clock started.");
        Ok(())
    }

    /// 3. AUTOMATED SETTLEMENT: Standard execution triggered by Consignee receipt.
    pub fn confirm_delivery_and_release_eur(ctx: Context<ConfirmDeliveryAndReleaseEur>) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow_account;

        require!(!escrow.is_completed, EscrowError::AlreadyCompleted);
        require!(!escrow.is_disputed, EscrowError::EscrowInDispute);
        require!(escrow.is_cmr_uploaded, EscrowError::CMRNotUploaded);
        require!(ctx.accounts.consignee.key() == escrow.consignee, EscrowError::UnauthorizedConsignee);

        disburse_funds(escrow, &ctx.accounts.eurc_vault, &ctx.accounts.carrier_eurc_account, &ctx.accounts.admin_fee_eurc_account, &ctx.accounts.payfreight_vault, &ctx.accounts.shipper_payfreight_account, &ctx.accounts.token_program, escrow.eurc_amount, 0)?;

        escrow.is_completed = true;
        msg!("e-CMR Delivery Confirmed by Consignee. Funds disbursed.");
        Ok(())
    }

    /// 4. TIMEOUT AUTO-RELEASE: Allows Carrier to claim funds if Consignee fails to respond in 14 days.
    /// @legal-note CMR Art. 30: Absence of reservations within the legal timeframe constitutes prima facie evidence of delivery.
    pub fn claim_timeout_release_eur(ctx: Context<ClaimTimeoutReleaseEur>) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow_account;

        require!(!escrow.is_completed, EscrowError::AlreadyCompleted);
        require!(!escrow.is_disputed, EscrowError::EscrowInDispute);
        require!(escrow.is_cmr_uploaded, EscrowError::CMRNotUploaded);
        require!(ctx.accounts.carrier.key() == escrow.carrier, EscrowError::UnauthorizedCarrier);

        let clock = Clock::get()?;
        let time_elapsed = clock.unix_timestamp - escrow.cmr_uploaded_at;
        require!(time_elapsed >= DISPUTE_WINDOW_SECONDS, EscrowError::DisputeWindowStillActive);

        disburse_funds(escrow, &ctx.accounts.eurc_vault, &ctx.accounts.carrier_eurc_account, &ctx.accounts.admin_fee_eurc_account, &ctx.accounts.payfreight_vault, &ctx.accounts.shipper_payfreight_account, &ctx.accounts.token_program, escrow.eurc_amount, 0)?;

        escrow.is_completed = true;
        msg!("Timeout period expired without dispute. Unilateral release executed.");
        Ok(())
    }

    /// 5. INITIATE DISPUTE: Shipper or Consignee can freeze funds in case of damage or missing cargo.
    /// @legal-note Freezes automated payouts pending formal cargo damage assessment under CMR Article 30.
    pub fn raise_dispute_eur(ctx: Context<RaiseDisputeEur>) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow_account;

        require!(!escrow.is_completed, EscrowError::AlreadyCompleted);
        require!(!escrow.is_disputed, EscrowError::AlreadyDisputed);
        
        let caller = ctx.accounts.caller.key();
        require!(caller == escrow.shipper || caller == escrow.consignee, EscrowError::UnauthorizedDisputeCaller);

        escrow.is_disputed = true;
        msg!("DISPUTE RAISED: On-chain settlement frozen. Referred to Arbitration.");
        Ok(())
    }

    /// 6. ARBITRATION RESOLUTION: Arbitrator divides funds based on settlement agreement / court order.
    /// @param carrier_share_eurc Amount allocated to Carrier after damage deductions.
    /// @param shipper_refund_eurc Amount refunded to Shipper for cargo loss/damage.
    pub fn resolve_dispute_eur(
        ctx: Context<ResolveDisputeEur>,
        carrier_share_eurc: u64,
        shipper_refund_eurc: u64,
    ) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow_account;

        require!(!escrow.is_completed, EscrowError::AlreadyCompleted);
        require!(escrow.is_disputed, EscrowError::NotInDispute);
        require!(ctx.accounts.arbitrator.key() == escrow.arbitrator, EscrowError::UnauthorizedArbitrator);
        require!(carrier_share_eurc + shipper_refund_eurc == escrow.eurc_amount, EscrowError::InvalidDisputeSplit);

        let order_id_bytes = escrow.order_id.as_bytes();
        let seeds = &[b"escrow_eur", order_id_bytes, &[escrow.bump]];
        let signer_seeds = &[&seeds[..]];
        let token_program = ctx.accounts.token_program.to_account_info();

        // 1. Payout Carrier share
        if carrier_share_eurc > 0 {
            let carrier_transfer = Transfer {
                from: ctx.accounts.eurc_vault.to_account_info(),
                to: ctx.accounts.carrier_eurc_account.to_account_info(),
                authority: escrow.to_account_info(),
            };
            token::transfer(CpiContext::new_with_signer(token_program.clone(), carrier_transfer, signer_seeds), carrier_share_eurc)?;
        }

        // 2. Refund Shipper share
        if shipper_refund_eurc > 0 {
            let shipper_refund = Transfer {
                from: ctx.accounts.eurc_vault.to_account_info(),
                to: ctx.accounts.shipper_eurc_account.to_account_info(),
                authority: escrow.to_account_info(),
            };
            token::transfer(CpiContext::new_with_signer(token_program.clone(), shipper_refund, signer_seeds), shipper_refund_eurc)?;
        }

        // 3. Return PAYFREIGHT collateral back to Shipper
        let payfreight_return = Transfer {
            from: ctx.accounts.payfreight_vault.to_account_info(),
            to: ctx.accounts.shipper_payfreight_account.to_account_info(),
            authority: escrow.to_account_info(),
        };
        token::transfer(CpiContext::new_with_signer(token_program, payfreight_return, signer_seeds), escrow.payfreight_amount)?;

        escrow.is_completed = true;
        msg!("Dispute legally resolved by Arbitrator. Funds split executed.");
        Ok(())
    }
}

// -----------------------------------------------------------------------------
// HELPER FUNCTIONS
// -----------------------------------------------------------------------------

fn disburse_funds<'info>(
    escrow: &Account<'info, EscrowAccountEur>,
    eurc_vault: &Account<'info, TokenAccount>,
    carrier_eurc: &Account<'info, TokenAccount>,
    admin_fee_eurc: &Account<'info, TokenAccount>,
    payfreight_vault: &Account<'info, TokenAccount>,
    shipper_payfreight: &Account<'info, TokenAccount>,
    token_program: &AccountInfo<'info>,
    total_eurc: u64,
    _deduction: u64,
) -> Result<()> {
    let fee_percentage = if escrow.payfreight_amount > 0 { 5 } else { 15 };
    let platform_fee = (total_eurc * fee_percentage) / 1000;
    let carrier_payout = total_eurc - platform_fee;

    let order_id_bytes = escrow.order_id.as_bytes();
    let seeds = &[b"escrow_eur", order_id_bytes, &[escrow.bump]];
    let signer_seeds = &[&seeds[..]];

    // Transfer Carrier Payout
    let carrier_transfer = Transfer {
        from: eurc_vault.to_account_info(),
        to: carrier_eurc.to_account_info(),
        authority: escrow.to_account_info(),
    };
    token::transfer(CpiContext::new_with_signer(token_program.clone(), carrier_transfer, signer_seeds), carrier_payout)?;

    // Transfer Platform Fee
    let fee_transfer = Transfer {
        from: eurc_vault.to_account_info(),
        to: admin_fee_eurc.to_account_info(),
        authority: escrow.to_account_info(),
    };
    token::transfer(CpiContext::new_with_signer(token_program.clone(), fee_transfer, signer_seeds), platform_fee)?;

    // Return PAYFREIGHT Collateral to Shipper
    let payfreight_return = Transfer {
        from: payfreight_vault.to_account_info(),
        to: shipper_payfreight.to_account_info(),
        authority: escrow.to_account_info(),
    };
    token::transfer(CpiContext::new_with_signer(token_program.clone(), payfreight_return, signer_seeds), escrow.payfreight_amount)?;

    Ok(())
}

// -----------------------------------------------------------------------------
// ACCOUNTS & CONTEXTS
// -----------------------------------------------------------------------------

#[derive(Accounts)]
#[instruction(order_id: String)]
pub struct InitializeEscrowEur<'info> {
    #[account(mut)]
    pub shipper: Signer<'info>,
    /// CHECK: Carrier public key
    pub carrier: AccountInfo<'info>,
    /// CHECK: Consignee public key
    pub consignee: AccountInfo<'info>,
    /// CHECK: Independent Arbitrator / Platform Legal Board public key
    pub arbitrator: AccountInfo<'info>,

    #[account(
        init,
        payer = shipper,
        space = 8 + 32 + 32 + 32 + 32 + 8 + 8 + 64 + 128 + 8 + 1 + 1 + 1 + 1,
        seeds = [b"escrow_eur", order_id.as_bytes()],
        bump
    )]
    pub escrow_account: Account<'info, EscrowAccountEur>,

    #[account(
        init,
        payer = shipper,
        seeds = [b"eurc_vault_eur", order_id.as_bytes()],
        bump,
        token::mint = eurc_mint,
        token::authority = escrow_account,
    )]
    pub eurc_vault: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = shipper,
        seeds = [b"payfreight_vault_eur", order_id.as_bytes()],
        bump,
        token::mint = payfreight_mint,
        token::authority = escrow_account,
    )]
    pub payfreight_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub shipper_eurc_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub shipper_payfreight_account: Account<'info, TokenAccount>,

    pub eurc_mint: Account<'info, Mint>,
    pub payfreight_mint: Account<'info, Mint>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct UploadCMREur<'info> {
    pub carrier: Signer<'info>,
    #[account(mut)]
    pub escrow_account: Account<'info, EscrowAccountEur>,
}

#[derive(Accounts)]
pub struct ConfirmDeliveryAndReleaseEur<'info> {
    pub consignee: Signer<'info>,
    #[account(mut, has_one = eurc_vault, has_one = payfreight_vault)]
    pub escrow_account: Account<'info, EscrowAccountEur>,
    #[account(mut)]
    pub eurc_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub payfreight_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub carrier_eurc_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub admin_fee_eurc_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub shipper_payfreight_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ClaimTimeoutReleaseEur<'info> {
    pub carrier: Signer<'info>,
    #[account(mut, has_one = eurc_vault, has_one = payfreight_vault)]
    pub escrow_account: Account<'info, EscrowAccountEur>,
    #[account(mut)]
    pub eurc_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub payfreight_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub carrier_eurc_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub admin_fee_eurc_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub shipper_payfreight_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct RaiseDisputeEur<'info> {
    pub caller: Signer<'info>,
    #[account(mut)]
    pub escrow_account: Account<'info, EscrowAccountEur>,
}

#[derive(Accounts)]
pub struct ResolveDisputeEur<'info> {
    pub arbitrator: Signer<'info>,
    #[account(mut, has_one = eurc_vault, has_one = payfreight_vault)]
    pub escrow_account: Account<'info, EscrowAccountEur>,
    #[account(mut)]
    pub eurc_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub payfreight_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub carrier_eurc_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub shipper_eurc_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub shipper_payfreight_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

// -----------------------------------------------------------------------------
// STATE STRUCT
// -----------------------------------------------------------------------------

#[account]
pub struct EscrowAccountEur {
    pub shipper: Pubkey,
    pub carrier: Pubkey,
    pub consignee: Pubkey,
    pub arbitrator: Pubkey,         // Arbitar za reševanje sporov
    pub eurc_amount: u64,
    pub payfreight_amount: u64,
    pub order_id: String,
    pub cmr_hash: String,
    pub cmr_uploaded_at: i64,       // Časovni žig za 14-dnevni rok
    pub is_cmr_uploaded: bool,
    pub is_completed: bool,
    pub is_disputed: bool,          // Zastavica za zamrznitev v primeru spora
    pub bump: u8,
}

// -----------------------------------------------------------------------------
// ERRORS
// -----------------------------------------------------------------------------

#[error_code]
pub enum EscrowError {
    #[msg("The escrow order has already been completed or released.")]
    AlreadyCompleted,
    #[msg("The escrow order is frozen due to an active dispute.")]
    EscrowInDispute,
    #[msg("The dispute window is still active. Consignee must be given time to respond.")]
    DisputeWindowStillActive,
    #[msg("The escrow order is not in a disputed state.")]
    NotInDispute,
    #[msg("A dispute is already active for this order.")]
    AlreadyDisputed,
    #[msg("The caller is not the designated carrier.")]
    UnauthorizedCarrier,
    #[msg("Only the designated consignee can confirm receipt.")]
    UnauthorizedConsignee,
    #[msg("Only Shipper or Consignee can raise a dispute.")]
    UnauthorizedDisputeCaller,
    #[msg("Only the designated Arbitrator can resolve this dispute.")]
    UnauthorizedArbitrator,
    #[msg("The e-CMR transport document has not been uploaded yet.")]
    CMRNotUploaded,
    #[msg("The split amounts do not equal the total escrow balance.")]
    InvalidDisputeSplit,
}
