//! =============================================================================
//! PAYFREIGHT ESCROW SMART CONTRACT (SOLANA / ANCHOR)
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
//!    the freight payment in USDC along with the required amount of $PAYFREIGHT tokens 
//!    (acting as collateral and a condition for lower platform fees).
//! 2. CMR Upload: The carrier fulfills the transport and uploads the hash of the 
//!    verified CMR document via the interface.
//! 3. Approval & Release: The administrator reviews the CMR and approves the payout. 
//!    The smart contract automatically transfers net USDC to the carrier, deducts a 0.5% 
//!    platform fee, and returns the locked $PAYFREIGHT tokens to the shipper.
//! =============================================================================

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

declare_id!("PayfrE1111111111111111111111111111111111111");

#[program]
pub mod payfreight_escrow {
    use super::*;

    /// 1. Shipper creates an order and locks USDC freight and $PAYFREIGHT tokens
    pub fn initialize_escrow(
        ctx: Context<InitializeEscrow>,
        order_id: String,
        usdc_amount: u64,
        payfreight_token_amount: u64,
    ) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow_account;
        escrow.shipper = ctx.accounts.shipper.key();
        escrow.carrier = ctx.accounts.carrier.key();
        escrow.admin = ctx.accounts.admin.key();
        escrow.usdc_amount = usdc_amount;
        escrow.payfreight_token_amount = payfreight_token_amount;
        escrow.order_id = order_id;
        escrow.cmr_hash = String::from("");
        escrow.is_cmr_uploaded = false;
        escrow.is_completed = false;
        escrow.bump = ctx.bumps.escrow_account;

        // A) Transfer USDC freight from shipper's account to the USDC Vault
        let cpi_accounts_usdc = Transfer {
            from: ctx.accounts.shipper_usdc_account.to_account_info(),
            to: ctx.accounts.usdc_vault.to_account_info(),
            authority: ctx.accounts.shipper.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        token::transfer(
            CpiContext::new(cpi_program.clone(), cpi_accounts_usdc),
            usdc_amount,
        )?;

        // B) Transfer $PAYFREIGHT tokens from shipper's account to the Token Vault
        let cpi_accounts_token = Transfer {
            from: ctx.accounts.shipper_payfreight_account.to_account_info(),
            to: ctx.accounts.payfreight_vault.to_account_info(),
            authority: ctx.accounts.shipper.to_account_info(),
        };
        token::transfer(
            CpiContext::new(cpi_program, cpi_accounts_token),
            payfreight_token_amount,
        )?;

        msg!("Escrow initialized: Locked {} USDC and {} $PAYFREIGHT.", usdc_amount, payfreight_token_amount);
        Ok(())
    }

    /// 2. Carrier uploads the verified CMR (document hash or URL)
    pub fn upload_cmr(ctx: Context<UploadCMR>, cmr_hash: String) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow_account;

        require!(!escrow.is_completed, EscrowError::AlreadyCompleted);
        require!(
            ctx.accounts.carrier.key() == escrow.carrier,
            EscrowError::UnauthorizedCarrier
        );

        escrow.cmr_hash = cmr_hash;
        escrow.is_cmr_uploaded = true;

        msg!("Verified CMR successfully uploaded to the system.");
        Ok(())
    }

    /// 3. Admin manually reviews CMR, releases USDC to carrier/platform, and returns tokens to shipper
    pub fn approve_and_release(ctx: Context<ApproveAndRelease>) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow_account;

        require!(!escrow.is_completed, EscrowError::AlreadyCompleted);
        require!(escrow.is_cmr_uploaded, EscrowError::CMRNotUploaded);
        require!(
            ctx.accounts.admin.key() == escrow.admin,
            EscrowError::UnauthorizedAdmin
        );

        // PDA signer seeds allowing the contract to autonomously release funds
        let order_id_bytes = escrow.order_id.as_bytes();
        let seeds = &[
            b"escrow",
            order_id_bytes,
            &[escrow.bump],
        ];
        let signer_seeds = &[&seeds[..]];

        // Fee calculation: 0.5% (50 basis points) for platform, 99.5% for carrier
        let platform_fee = (escrow.usdc_amount * 50) / 10000;
        let carrier_payout = escrow.usdc_amount - platform_fee;

        let cpi_program = ctx.accounts.token_program.to_account_info();

        // A) Transfer net freight to Carrier (USDC)
        let transfer_to_carrier = Transfer {
            from: ctx.accounts.usdc_vault.to_account_info(),
            to: ctx.accounts.carrier_usdc_account.to_account_info(),
            authority: escrow.to_account_info(),
        };
        token::transfer(
            CpiContext::new_with_signer(cpi_program.clone(), transfer_to_carrier, signer_seeds),
            carrier_payout,
        )?;

        // B) Transfer platform fee to Payfreight platform wallet (USDC)
        let transfer_fee = Transfer {
            from: ctx.accounts.usdc_vault.to_account_info(),
            to: ctx.accounts.admin_fee_usdc_account.to_account_info(),
            authority: escrow.to_account_info(),
        };
        token::transfer(
            CpiContext::new_with_signer(cpi_program.clone(), transfer_fee, signer_seeds),
            platform_fee,
        )?;

        // C) Return locked $PAYFREIGHT tokens back to the Shipper
        let return_tokens = Transfer {
            from: ctx.accounts.payfreight_vault.to_account_info(),
            to: ctx.accounts.shipper_payfreight_account.to_account_info(),
            authority: escrow.to_account_info(),
        };
        token::transfer(
            CpiContext::new_with_signer(cpi_program, return_tokens, signer_seeds),
            escrow.payfreight_token_amount,
        )?;

        escrow.is_completed = true;

        msg!("Payment released! Carrier: {} USDC, Fee: {} USDC, Tokens returned to shipper.", carrier_payout, platform_fee);
        Ok(())
    }
}

// -----------------------------------------------------------------------------
// ACCOUNTS & CONTEXTS
// -----------------------------------------------------------------------------

#[derive(Accounts)]
#[instruction(order_id: String)]
pub struct InitializeEscrow<'info> {
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
        seeds = [b"escrow", order_id.as_bytes()],
        bump
    )]
    pub escrow_account: Account<'info, EscrowAccount>,

    // Vault for USDC freight
    #[account(
        init,
        payer = shipper,
        seeds = [b"usdc_vault", order_id.as_bytes()],
        bump,
        token::mint = usdc_mint,
        token::authority = escrow_account,
    )]
    pub usdc_vault: Account<'info, TokenAccount>,

    // Vault for $PAYFREIGHT tokens
    #[account(
        init,
        payer = shipper,
        seeds = [b"payfreight_vault", order_id.as_bytes()],
        bump,
        token::mint = payfreight_mint,
        token::authority = escrow_account,
    )]
    pub payfreight_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub shipper_usdc_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub shipper_payfreight_account: Account<'info, TokenAccount>,

    pub usdc_mint: Account<'info, token::Mint>,
    pub payfreight_mint: Account<'info, token::Mint>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct UploadCMR<'info> {
    pub carrier: Signer<'info>,
    #[account(mut)]
    pub escrow_account: Account<'info, EscrowAccount>,
}

#[derive(Accounts)]
pub struct ApproveAndRelease<'info> {
    pub admin: Signer<'info>,

    #[account(
        mut,
        has_one = usdc_vault,
        has_one = payfreight_vault,
    )]
    pub escrow_account: Account<'info, EscrowAccount>,

    #[account(mut)]
    pub usdc_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub payfreight_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub carrier_usdc_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub admin_fee_usdc_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub shipper_payfreight_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

// -----------------------------------------------------------------------------
// ESCROW STATE
// -----------------------------------------------------------------------------

#[account]
pub struct EscrowAccount {
    pub shipper: Pubkey,                    // Shipper / Client
    pub carrier: Pubkey,                    // Carrier / Transporter
    pub admin: Pubkey,                      // Administrator
    pub usdc_amount: u64,                   // Freight amount in USDC
    pub payfreight_token_amount: u64,     // Locked $PAYFREIGHT token amount
    pub order_id: String,                   // Order ID or Invoice Number
    pub cmr_hash: String,                   // Link/Hash of verified CMR
    pub is_cmr_uploaded: bool,            // CMR upload status flag
    pub is_completed: bool,                 // Transaction completion flag
    pub bump: u8,                           // PDA Bump
}

// -----------------------------------------------------------------------------
// ERROR CODES
// -----------------------------------------------------------------------------

#[error_code]
pub enum EscrowError {
    #[msg("The order has already been completed.")]
    AlreadyCompleted,
    #[msg("Only the designated carrier can upload the CMR.")]
    UnauthorizedCarrier,
    #[msg("Only the authorized administrator can approve the payout.")]
    UnauthorizedAdmin,
    #[msg("The CMR document has not been uploaded yet.")]
    CMRNotUploaded,
}
