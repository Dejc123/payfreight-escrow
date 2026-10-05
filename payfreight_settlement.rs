/*
 * Copyright (c) 2026 PayFreight. All rights reserved.
 * Author: PayFreight Protocol Team
 * 
 * =========================================================================================
 * PAYFREIGHT PROTOCOL - SOLANA USD SETTLEMENT PROGRAM (ANCHOR)
 * E-CMR COMPLIANCE, AUTOMATED SETTLEMENT & DISPUTE ARBITRATION MODULE (USDC / USDT)
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
 * 
 * TOKEN UTILITY & DYNAMIC FEE DISCOUNT TIER:
 * - Neither holds utility tokens: 1.5% (150 bps)
 * - Single party holds utility tokens: 1.0% (100 bps)
 * - Both parties hold utility tokens: 0.5% (50 bps)
 */

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

declare_id!("PayFreightProtocolUsd111111111111111111111");

pub const DISPUTE_WINDOW_SECONDS: i64 = 14 * 24 * 60 * 60; // 14 days legal claim window

#[program]
pub mod payfreight_settlement_usd {
    use super::*;

    /// 1. Shipper initializes the Escrow, registers Shipper, Carrier, Consignee, and Arbitrator,
    ///    and locks USDC/USDT stablecoins into the escrow vault PDA.
    pub fn create_freight_escrow_usd(
        ctx: Context<CreateEscrowUsd>,
        freight_id: String,
        amount_usdc: u64,
    ) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow_account;
        escrow.shipper = ctx.accounts.shipper.key();
        escrow.carrier = ctx.accounts.carrier.key();
        escrow.consignee = ctx.accounts.consignee.key();
        escrow.arbitrator = ctx.accounts.arbitrator.key();
        escrow.amount = amount_usdc;
        escrow.freight_id = freight_id;
        escrow.cmr_hash = String::from("");
        escrow.is_cmr_uploaded = false;
        escrow.cmr_uploaded_at = 0;
        escrow.is_settled = false;
        escrow.is_disputed = false;
        escrow.bump = ctx.bumps.escrow_account;

        // Transfer USDC/USDT from shipper wallet to the escrow vault PDA
        let cpi_accounts = Transfer {
            from: ctx.accounts.shipper_usdc.to_account_info(),
            to: ctx.accounts.escrow_vault.to_account_info(),
            authority: ctx.accounts.shipper.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        token::transfer(CpiContext::new(cpi_program, cpi_accounts), amount_usdc)?;

        msg!("USD Freight Escrow initialized with Arbitration fallback enabled.");
        Ok(())
    }

    /// 2. Carrier uploads the cryptographically hashed e-CMR document (SHA-256).
    /// @legal-note Proof of transport initialization and carriage details as per e-CMR Article 6.
    pub fn upload_cmr_usd(ctx: Context<UploadCMRUsd>, cmr_hash: String) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow_account;

        require!(!escrow.is_settled, CustomError::AlreadySettled);
        require!(!escrow.is_disputed, CustomError::EscrowInDispute);
        require!(ctx.accounts.carrier.key() == escrow.carrier, CustomError::UnauthorizedCarrier);

        let clock = Clock::get()?;
        escrow.cmr_hash = cmr_hash;
        escrow.is_cmr_uploaded = true;
        escrow.cmr_uploaded_at = clock.unix_timestamp;

        msg!("e-CMR document hash anchored on-chain for USD settlement.");
        Ok(())
    }

    /// 3. AUTOMATED SETTLEMENT TRIGGER: Consignee digital sign-off and payment release.
    /// 
    /// JUDICIAL STATEMENT FOR REGULATORY & LEGAL REVIEW:
    /// Pursuant to the UN/ECE e-CMR Protocol (Article 9 - Receipt of the goods), the carriage 
    /// contract is fully executed upon the consignee's receipt and verification of the cargo.
    /// This function acts as a self-executing smart contract trigger (Escrow Settlement).
    pub fn confirm_delivery_and_release_usd(
        ctx: Context<ConfirmDeliveryAndReleaseUsd>,
        shipper_has_tokens: bool,
        carrier_has_tokens: bool,
    ) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow_account;

        require!(!escrow.is_settled, CustomError::AlreadySettled);
        require!(!escrow.is_disputed, CustomError::EscrowInDispute);
        require!(escrow.is_cmr_uploaded, CustomError::CMRNotUploaded);
        require!(ctx.accounts.consignee.key() == escrow.consignee, CustomError::UnauthorizedConsignee);

        disburse_usd_funds(
            escrow,
            &ctx.accounts.escrow_vault,
            &ctx.accounts.carrier_usdc,
            &ctx.accounts.admin_fee_usdc,
            &ctx.accounts.token_program,
            escrow.amount,
            shipper_has_tokens,
            carrier_has_tokens,
        )?;

        escrow.is_settled = true;
        msg!("e-CMR Delivery Confirmed by Consignee. USD Funds disbursed.");
        Ok(())
    }

    /// 4. TIMEOUT AUTO-RELEASE: Allows Carrier to claim funds if Consignee fails to respond within 14 days.
    /// @legal-note CMR Art. 30: Absence of reservations within the legal timeframe constitutes prima facie evidence of delivery.
    pub fn claim_timeout_release_usd(
        ctx: Context<ClaimTimeoutReleaseUsd>,
        shipper_has_tokens: bool,
        carrier_has_tokens: bool,
    ) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow_account;

        require!(!escrow.is_settled, CustomError::AlreadySettled);
        require!(!escrow.is_disputed, CustomError::EscrowInDispute);
        require!(escrow.is_cmr_uploaded, CustomError::CMRNotUploaded);
        require!(ctx.accounts.carrier.key() == escrow.carrier, CustomError::UnauthorizedCarrier);

        let clock = Clock::get()?;
        let time_elapsed = clock.unix_timestamp - escrow.cmr_uploaded_at;
        require!(time_elapsed >= DISPUTE_WINDOW_SECONDS, CustomError::DisputeWindowStillActive);

        disburse_usd_funds(
            escrow,
            &ctx.accounts.escrow_vault,
            &ctx.accounts.carrier_usdc,
            &ctx.accounts.admin_fee_usdc,
            &ctx.accounts.token_program,
            escrow.amount,
            shipper_has_tokens,
            carrier_has_tokens,
        )?;

        escrow.is_settled = true;
        msg!("Timeout period expired without dispute. Unilateral USD release executed.");
        Ok(())
    }

    /// 5. INITIATE DISPUTE: Shipper or Consignee can freeze funds in case of damage or missing cargo.
    /// @legal-note Freezes automated payouts pending formal cargo damage assessment under CMR Article 30.
    pub fn raise_dispute_usd(ctx: Context<RaiseDisputeUsd>) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow_account;

        require!(!escrow.is_settled, CustomError::AlreadySettled);
        require!(!escrow.is_disputed, CustomError::AlreadyDisputed);

        let caller = ctx.accounts.caller.key();
        require!(
            caller == escrow.shipper || caller == escrow.consignee,
            CustomError::UnauthorizedDisputeCaller
        );

        escrow.is_disputed = true;
        msg!("USD DISPUTE RAISED: On-chain settlement frozen. Referred to Arbitration.");
        Ok(())
    }

    /// 6. ARBITRATION RESOLUTION: Arbitrator divides funds based on settlement agreement / court order.
    pub fn resolve_dispute_usd(
        ctx: Context<ResolveDisputeUsd>,
        carrier_share_usdc: u64,
        shipper_refund_usdc: u64,
    ) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow_account;

        require!(!escrow.is_settled, CustomError::AlreadySettled);
        require!(escrow.is_disputed, CustomError::NotInDispute);
        require!(ctx.accounts.arbitrator.key() == escrow.arbitrator, CustomError::UnauthorizedArbitrator);
        require!(carrier_share_usdc + shipper_refund_usdc == escrow.amount, CustomError::InvalidDisputeSplit);

        let freight_id_bytes = escrow.freight_id.as_bytes();
        let seeds = &[b"escrow_usd", freight_id_bytes, &[escrow.bump]];
        let signer_seeds = &[&seeds[..]];
        let token_program = ctx.accounts.token_program.to_account_info();

        // 1. Payout Carrier share
        if carrier_share_usdc > 0 {
            let carrier_transfer = Transfer {
                from: ctx.accounts.escrow_vault.to_account_info(),
                to: ctx.accounts.carrier_usdc.to_account_info(),
                authority: escrow.to_account_info(),
            };
            token::transfer(CpiContext::new_with_signer(token_program.clone(), carrier_transfer, signer_seeds), carrier_share_usdc)?;
        }

        // 2. Refund Shipper share
        if shipper_refund_usdc > 0 {
            let shipper_refund = Transfer {
                from: ctx.accounts.escrow_vault.to_account_info(),
                to: ctx.accounts.shipper_usdc.to_account_info(),
                authority: escrow.to_account_info(),
            };
            token::transfer(CpiContext::new_with_signer(token_program, shipper_refund, signer_seeds), shipper_refund_usdc)?;
        }

        escrow.is_settled = true;
        msg!("USD Dispute legally resolved by Arbitrator. Split executed.");
        Ok(())
    }
}

// -----------------------------------------------------------------------------
// HELPER FUNCTIONS
// -----------------------------------------------------------------------------

fn disburse_usd_funds<'info>(
    escrow: &Account<'info, FreightEscrow>,
    escrow_vault: &Account<'info, TokenAccount>,
    carrier_usdc: &Account<'info, TokenAccount>,
    admin_fee_usdc: &Account<'info, TokenAccount>,
    token_program: &AccountInfo<'info>,
    total_usdc: u64,
    shipper_has_tokens: bool,
    carrier_has_tokens: bool,
) -> Result<()> {
    // Dynamic Fee Calculation
    let fee_bps: u64 = match (shipper_has_tokens, carrier_has_tokens) {
        (true, true) => 50,                  // 0.5%
        (true, false) | (false, true) => 100, // 1.0%
        (false, false) => 150,               // 1.5%
    };

    let platform_fee = (total_usdc * fee_bps) / 10000;
    let carrier_payout = total_usdc - platform_fee;

    let freight_id_bytes = escrow.freight_id.as_bytes();
    let seeds = &[b"escrow_usd", freight_id_bytes, &[escrow.bump]];
    let signer_seeds = &[&seeds[..]];

    // 1. Transfer Payout to Carrier
    let carrier_transfer = Transfer {
        from: escrow_vault.to_account_info(),
        to: carrier_usdc.to_account_info(),
        authority: escrow.to_account_info(),
    };
    token::transfer(
        CpiContext::new_with_signer(token_program.clone(), carrier_transfer, signer_seeds),
        carrier_payout,
    )?;

    // 2. Transfer Fee to Platform Treasury
    let fee_transfer = Transfer {
        from: escrow_vault.to_account_info(),
        to: admin_fee_usdc.to_account_info(),
        authority: escrow.to_account_info(),
    };
    token::transfer(
        CpiContext::new_with_signer(token_program.clone(), fee_transfer, signer_seeds),
        platform_fee,
    )?;

    Ok(())
}

// -----------------------------------------------------------------------------
// ACCOUNTS & CONTEXTS
// -----------------------------------------------------------------------------

#[derive(Accounts)]
#[instruction(freight_id: String)]
pub struct CreateEscrowUsd<'info> {
    #[account(
        init,
        payer = shipper,
        space = 8 + 32 + 32 + 32 + 32 + 8 + 64 + 128 + 8 + 1 + 1 + 1 + 1,
        seeds = [b"escrow_usd", freight_id.as_bytes()],
        bump
    )]
    pub escrow_account: Account<'info, FreightEscrow>,

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
        seeds = [b"usdc_vault", freight_id.as_bytes()],
        bump,
        token::mint = usdc_mint,
        token::authority = escrow_account,
    )]
    pub escrow_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub shipper_usdc: Account<'info, TokenAccount>,

    pub usdc_mint: Account<'info, Mint>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct UploadCMRUsd<'info> {
    pub carrier: Signer<'info>,
    #[account(mut)]
    pub escrow_account: Account<'info, FreightEscrow>,
}

#[derive(Accounts)]
pub struct ConfirmDeliveryAndReleaseUsd<'info> {
    pub consignee: Signer<'info>,
    #[account(mut, has_one = escrow_vault)]
    pub escrow_account: Account<'info, FreightEscrow>,
    #[account(mut)]
    pub escrow_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub carrier_usdc: Account<'info, TokenAccount>,
    #[account(mut)]
    pub admin_fee_usdc: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ClaimTimeoutReleaseUsd<'info> {
    pub carrier: Signer<'info>,
    #[account(mut, has_one = escrow_vault)]
    pub escrow_account: Account<'info, FreightEscrow>,
    #[account(mut)]
    pub escrow_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub carrier_usdc: Account<'info, TokenAccount>,
    #[account(mut)]
    pub admin_fee_usdc: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct RaiseDisputeUsd<'info> {
    pub caller: Signer<'info>,
    #[account(mut)]
    pub escrow_account: Account<'info, FreightEscrow>,
}

#[derive(Accounts)]
pub struct ResolveDisputeUsd<'info> {
    pub arbitrator: Signer<'info>,
    #[account(mut, has_one = escrow_vault)]
    pub escrow_account: Account<'info, FreightEscrow>,
    #[account(mut)]
    pub escrow_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub carrier_usdc: Account<'info, TokenAccount>,
    #[account(mut)]
    pub shipper_usdc: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

// -----------------------------------------------------------------------------
// STATE STRUCT
// -----------------------------------------------------------------------------

#[account]
pub struct FreightEscrow {
    pub shipper: Pubkey,
    pub carrier: Pubkey,
    pub consignee: Pubkey,
    pub arbitrator: Pubkey,
    pub amount: u64,
    pub freight_id: String,
    pub cmr_hash: String,
    pub cmr_uploaded_at: i64,
    pub is_cmr_uploaded: bool,
    pub is_settled: bool,
    pub is_disputed: bool,
    pub bump: u8,
}

// -----------------------------------------------------------------------------
// ERRORS
// -----------------------------------------------------------------------------

#[error_code]
pub enum CustomError {
    #[msg("This freight payment has already been settled.")]
    AlreadySettled,
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
}
