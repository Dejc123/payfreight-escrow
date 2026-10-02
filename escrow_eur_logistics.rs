/*
 * Copyright (c) 2026 PayFreight. All rights reserved.
 * Author: Dejc123 & Team
 * 
 * This source code is proprietary and confidential. 
 * Unauthorized copying of this file, via any medium, is strictly prohibited.
*/

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, Token, TokenAccount, Transfer};

declare_id!("PayfrE2222222222222222222222222222222222222");

#[program]
pub mod payfreight_escrow_eur {
    use super::*;

    /// 1. Shipper initializes the Escrow with transport fare, fixed cancellation (250 EUR), 
    /// 3 days waiting fee (600 EUR), and locks PAYFREIGHT tokens for staking.
    pub fn initialize_escrow_eur(
        ctx: Context<InitializeEscrowEur>,
        order_id: String,
        transport_fare: u64,
        payfreight_amount: u64,
    ) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow_account;
        escrow.shipper = ctx.accounts.shipper.key();
        escrow.carrier = ctx.accounts.carrier.key();
        escrow.admin = ctx.accounts.admin.key();
        escrow.transport_fare = transport_fare;
        escrow.payfreight_amount = payfreight_amount;
        escrow.order_id = order_id;
        escrow.cmr_hash = String::from("");
        escrow.is_cmr_uploaded = false;
        escrow.is_completed = false;
        escrow.bump = ctx.bumps.escrow_account;

        // Total EURC deposit = Transport Fare + 250 EUR (cancellation) + 600 EUR (3 days waiting fee)
        let total_eurc_deposit = transport_fare + 250_000_000 + 600_000_000; // Assuming 6 decimals for EURC

        // Transfer total EURC from shipper to EURC Vault PDA
        let cpi_eurc_accounts = Transfer {
            from: ctx.accounts.shipper_eurc_account.to_account_info(),
            to: ctx.accounts.eurc_vault.to_account_info(),
            authority: ctx.accounts.shipper.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        token::transfer(
            CpiContext::new(cpi_program.clone(), cpi_eurc_accounts),
            total_eurc_deposit,
        )?;

        // Transfer PAYFREIGHT staking tokens from shipper to PAYFREIGHT Vault PDA
        let cpi_payfreight_accounts = Transfer {
            from: ctx.accounts.shipper_payfreight_account.to_account_info(),
            to: ctx.accounts.payfreight_vault.to_account_info(),
            authority: ctx.accounts.shipper.to_account_info(),
        };
        token::transfer(
            CpiContext::new(cpi_program, cpi_payfreight_accounts),
            payfreight_amount,
        )?;

        msg!("EUR Escrow initialized with strict logistics rules and staking.");
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

    /// 3. Admin resolves the deal based on CMR data (Waiting days 0-3 or Carrier Cancellation)
    pub fn resolve_deal_eur(
        ctx: Context<ResolveDealEur>,
        waiting_days: u8,
        is_carrier_no_show: bool,
    ) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow_account;

        require!(!escrow.is_completed, EscrowError::AlreadyCompleted);
        require!(escrow.is_cmr_uploaded, EscrowError::CMRNotUploaded);
        require!(
            ctx.accounts.admin.key() == escrow.admin,
            EscrowError::UnauthorizedAdmin
        );
        require!(waiting_days <= 3, EscrowError::InvalidWaitingDays);

        let token_program = ctx.accounts.token_program.to_account_info();

        // PDA Signer Seeds
        let order_id_bytes = escrow.order_id.as_bytes();
        let seeds = &[
            b"escrow_eur",
            order_id_bytes,
            &[escrow.bump],
        ];
        let signer_seeds = &[&seeds[..]];

        // Constant values (scaled to 6 decimals: 250 EUR = 250_000_000, 200 EUR = 200_000_000)
        let cancellation_fee = 250_000_000;
        let daily_waiting_fee = 200_000_000;
        let total_waiting_pool = 600_000_000;

        if is_carrier_no_show {
            // SCENARIO C: Carrier didn't show up on loading day.
            // Shipper gets back: Transport fare + 850 EUR (250 cancellation + 600 waiting) + 250 penalty from carrier.
            let total_shipper_refund = escrow.transport_fare + cancellation_fee + total_waiting_pool;

            let refund_transfer = Transfer {
                from: ctx.accounts.eurc_vault.to_account_info(),
                to: ctx.accounts.shipper_eurc_account.to_account_info(),
                authority: escrow.to_account_info(),
            };
            token::transfer(
                CpiContext::new_with_signer(token_program.clone(), refund_transfer, signer_seeds),
                total_shipper_refund,
            )?;

            let penalty_transfer = Transfer {
                from: ctx.accounts.carrier_eurc_account.to_account_info(),
                to: ctx.accounts.shipper_eurc_account.to_account_info(),
                authority: ctx.accounts.carrier.to_account_info(),
            };
            token::transfer(
                CpiContext::new(token_program.clone(), penalty_transfer),
                cancellation_fee,
            )?;

        } else {
            // SCENARIOS A & B: Normal delivery or waiting days (1, 2, or 3 days)
            let waiting_payout = (waiting_days as u64) * daily_waiting_fee;
            let carrier_total_payout = escrow.transport_fare + waiting_payout;

            let unused_waiting_pool = total_waiting_pool - waiting_payout;
            let shipper_refund = cancellation_fee + unused_waiting_pool;

            let carrier_transfer = Transfer {
                from: ctx.accounts.eurc_vault.to_account_info(),
                to: ctx.accounts.carrier_eurc_account.to_account_info(),
                authority: escrow.to_account_info(),
            };
            token::transfer(
                CpiContext::new_with_signer(token_program.clone(), carrier_transfer, signer_seeds),
                carrier_total_payout,
            )?;

            if shipper_refund > 0 {
                let shipper_transfer = Transfer {
                    from: ctx.accounts.eurc_vault.to_account_info(),
                    to: ctx.accounts.shipper_eurc_account.to_account_info(),
                    authority: escrow.to_account_info(),
                };
                token::transfer(
                    CpiContext::new_with_signer(token_program.clone(), shipper_transfer, signer_seeds),
                    shipper_refund,
                )?;
            }
        }

        // Return staked PAYFREIGHT tokens back to Shipper
        let payfreight_return = Transfer {
            from: ctx.accounts.payfreight_vault.to_account_info(),
            to: ctx.accounts.shipper_payfreight_account.to_account_info(),
            authority: escrow.to_account_info(),
        };
        token::transfer(
            CpiContext::new_with_signer(token_program.clone(), payfreight_return, signer_seeds),
            escrow.payfreight_amount,
        )?;

        // =====================================================================
        // TOKENOMICS & DYNAMIC DEMAND SCALING (BITCOIN-STYLE HALVING MODEL)
        // =====================================================================
        // Note on Long-Term Tokenomics:
        // As platform demand increases over the years, the required staking amount 
        // and burn fees are structured to decrease periodically (similar to Bitcoin's 
        // 4-year halving cycle) to prevent token price appreciation from pricing out carriers:
        // - Years 0-4:  10,000 PAYF stake / 10 PAYF burn per deal
        // - Years 4-8:   5,000 PAYF stake /  5 PAYF burn per deal
        // - Years 8-12:  2,500 PAYF stake /  2 PAYF burn per deal
        // This ensures the ecosystem remains accessible and fluid at scale.
        // =====================================================================

        let burn_amount = 10_000_000; // Assuming 6 decimals for PAYF token
        let burn_cpi = Burn {
            mint: ctx.accounts.payfreight_mint.to_account_info(),
            from: ctx.accounts.payfreight_vault.to_account_info(),
            authority: escrow.to_account_info(),
        };
        let _ = token::burn(
            CpiContext::new_with_signer(token_program, burn_cpi, signer_seeds),
            burn_amount,
        );

        escrow.is_completed = true;
        msg!("Escrow successfully resolved and closed according to logistics rules. 10 PAYF burned.");
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
pub struct ResolveDealEur<'info> {
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
    pub shipper_eurc_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub payfreight_mint: Account<'info, Mint>,
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
    pub transport_fare: u64,
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
    #[msg("Invalid waiting days specified (must be between 0 and 3).")]
    InvalidWaitingDays,
}
