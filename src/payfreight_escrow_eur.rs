//! =============================================================================
//! PAYFREIGHT EUR ESCROW SMART CONTRACT (SOLANA / ANCHOR)
//! =============================================================================
//! 
//! Copyright (c) 2026 Payfreight / All Rights Reserved.
//! 
//! Hackathon judges and evaluators are granted free access to view, analyze,
//! and test this code exclusively for the purpose of evaluating this project.
//! Any other use, copying, distribution, modification, or commercial exploitation 
//! of this code without explicit written permission from the author is strictly prohibited.
//! 
//! =============================================================================
//! BUSINESS LOGIC & WORKFLOW DESCRIPTION:
//! 1. Escrow Initialization: The shipper creates an order and simultaneously locks 
//!    the freight payment in EURC along with the required amount of $PAYFREIGHT tokens 
//!    (acting as collateral and a condition for lower platform fees).
//! 2. CMR Upload: The carrier fulfills the transport and uploads the hash of the 
//!    verified e-CMR document via the interface.
//! 3. Approval & Release: The administrator reviews the e-CMR and approves the payout. 
//!    The smart contract automatically applies dynamic fees (0.5% if $PAYFREIGHT tokens 
//!    are locked, or 1.5% standard fee if not), transfers net EURC to the carrier, collects 
//!    the platform fee, and returns the locked $PAYFREIGHT tokens to the shipper.
//! =============================================================================

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

declare_id!("PayfrE2222222222222222222222222222222222222"); // Unique Program ID for EUR version

#[program]
pub mod payfreight_escrow_eur {
    use super::*;

    /// 1. Shipper initializes the Escrow and locks EURC and PAYFREIGHT tokens
    pub fn initialize_escrow_eur(
        ctx: Context<InitializeEscrowEur>,
        order_id: String,
        eurc_amount: u64,
        payfreight_amount: u64,
    ) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow_account;
        escrow.shipper = ctx.accounts.shipper.key();
        escrow.carrier = ctx.accounts.carrier.key();
        escrow.admin = ctx.accounts.admin.key();
        escrow.eurc_amount = eurc_amount;
        escrow.payfreight_amount = payfreight_amount;
        escrow.order_id = order_id;
        escrow.cmr_hash = String::from("");
        escrow.is_cmr_uploaded = false;
        escrow.is_completed = false;
        escrow.bump = ctx.bumps.escrow_account;

        // Transfer EURC from shipper to EURC Vault PDA
        let cpi_eurc_accounts = Transfer {
            from: ctx.accounts.shipper_eurc_account.to_account_info(),
            to: ctx.accounts.eurc_vault.to_account_info(),
            authority: ctx.accounts.shipper.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_eurc_ctx = CpiContext::new(cpi_program.clone(), cpi_eurc_accounts);
        token::transfer(cpi_eurc_ctx, eurc_amount)?;

        // Transfer PAYFREIGHT utility tokens from shipper to PAYFREIGHT Vault PDA
        let cpi_payfreight_accounts = Transfer {
            from: ctx.accounts.shipper_payfreight_account.to_account_info(),
            to: ctx.accounts.payfreight_vault.to_account_info(),
            authority: ctx.accounts.shipper.to_account_info(),
        };
        let cpi_payfreight_ctx = CpiContext::new(cpi_program, cpi_payfreight_accounts);
        token::transfer(cpi_payfreight_ctx, payfreight_amount)?;

        msg!("EUR Escrow initialized successfully. EURC and PAYFREIGHT locked in vaults.");
        Ok(())
    }

    /// 2. Carrier uploads the signed e-CMR document hash
    pub fn upload_cmr_eur(ctx: Context<UploadCMREur>, cmr_hash: String) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow_account;

        require!(!escrow.is_completed, EscrowError::AlreadyCompleted);
        require!(
            ctx.accounts.carrier.key() == escrow.carrier,
            EscrowError::UnauthorizedCarrier
        );

        escrow.cmr_hash = cmr_hash;
        escrow.is_cmr_uploaded = true;

        msg!("e-CMR document successfully uploaded.");
        Ok(())
    }

    /// 3. Admin verifies CMR, calculates dynamic fee, and releases funds / tokens
    pub fn approve_and_release_eur(ctx: Context<ApproveAndReleaseEur>) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow_account;

        require!(!escrow.is_completed, EscrowError::AlreadyCompleted);
        require!(escrow.is_cmr_uploaded, EscrowError::CMRNotUploaded);
        require!(
            ctx.accounts.admin.key() == escrow.admin,
            EscrowError::UnauthorizedAdmin
        );

        // Dynamic Fee Logic: 0.5% fee if PAYFREIGHT tokens are locked, 1.5% standard fee otherwise
        let fee_percentage = if escrow.payfreight_amount > 0 { 5 } else { 15 }; // Basis points scaling (5 = 0.5%, 15 = 1.5%)
        let platform_fee = (escrow.eurc_amount * fee_percentage) / 1000;
        let carrier_payout = escrow.eurc_amount - platform_fee;

        // PDA Signer Seeds
        let order_id_bytes = escrow.order_id.as_bytes();
        let seeds = &[
            b"escrow_eur",
            order_id_bytes,
            &[escrow.bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let token_program = ctx.accounts.token_program.to_account_info();

        // Transfer net EURC to Carrier
        let carrier_transfer = Transfer {
            from: ctx.accounts.eurc_vault.to_account_info(),
            to: ctx.accounts.carrier_eurc_account.to_account_info(),
            authority: escrow.to_account_info(),
        };
        token::transfer(
            CpiContext::new_with_signer(token_program.clone(), carrier_transfer, signer_seeds),
            carrier_payout,
        )?;

        // Transfer Platform Fee to Admin Treasury
        let fee_transfer = Transfer {
            from: ctx.accounts.eurc_vault.to_account_info(),
            to: ctx.accounts.admin_fee_eurc_account.to_account_info(),
            authority: escrow.to_account_info(),
        };
        token::transfer(
            CpiContext::new_with_signer(token_program.clone(), fee_transfer, signer_seeds),
            platform_fee,
        )?;

        // Return locked PAYFREIGHT tokens back to the Shipper
        let payfreight_return = Transfer {
            from: ctx.accounts.payfreight_vault.to_account_info(),
            to: ctx.accounts.shipper_payfreight_account.to_account_info(),
            authority: escrow.to_account_info(),
        };
        token::transfer(
            CpiContext::new_with_signer(token_program, payfreight_return, signer_seeds),
            escrow.payfreight_amount,
        )?;

        escrow.is_completed = true;

        msg!("EUR Escrow successfully completed. Payout and fee distributed.");
        Ok(())
    }
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
    /// CHECK: Admin public key
    pub admin: AccountInfo<'info>,

    #[account(
        init,
        payer = shipper,
        space = 8 + 32 + 32 + 32 + 8 + 8 + 64 + 128 + 1 + 1 + 1,
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
pub struct ApproveAndReleaseEur<'info> {
    pub admin: Signer<'info>,
    #[account(
        mut,
        has_one = eurc_vault,
        has_one = payfreight_vault,
    )]
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

// -----------------------------------------------------------------------------
// STATE STRUCT
// -----------------------------------------------------------------------------

#[account]
pub struct EscrowAccountEur {
    pub shipper: Pubkey,
    pub carrier: Pubkey,
    pub admin: Pubkey,
    pub eurc_amount: u64,
    pub payfreight_amount: u64,
    pub order_id: String,
    pub cmr_hash: String,
    pub is_cmr_uploaded: bool,
    pub is_completed: bool,
    pub bump: u8,
}

// -----------------------------------------------------------------------------
// ERRORS
// -----------------------------------------------------------------------------

#[error_code]
pub enum EscrowError {
    #[msg("The escrow order has already been completed or released.")]
    AlreadyCompleted,
    #[msg("The caller is not the designated carrier for this order.")]
    UnauthorizedCarrier,
    #[msg("Only the designated administrator can approve and release funds.")]
    UnauthorizedAdmin,
    #[msg("The e-CMR transport document has not been uploaded yet.")]
    CMRNotUploaded,
}
