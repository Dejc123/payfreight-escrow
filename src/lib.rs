//! =============================================================================
//! PAYFREIGHT ESCROW SMART CONTRACT (SOLANA / ANCHOR)
//! =============================================================================
//! 
//! Copyright (c) 2026 Payfreight / Vse pravice pridržane.
//! 
//! Ocenjevalci in sodniki dogodka/hackathona si lahko to kodo prosto ogledajo,
//! jo analizirajo in preizkusijo izključno za namene ocenjevanja tega projekta.
//! Kakršna koli drugačna uporaba, kopiranje, distribucija, spreminjanje ali
//! komercialna raba te kode brez izrecnega pisnega dovoljenja avtorja ni dovoljena.
//! 
//! =============================================================================
//! OPIS DELOVANJA IN POSLOVNI MODEL:
//! 1. Zaklep (Escrow): Špediter ustvari naročilo ter v sefe hkrati zaklene 
//!    voznino v USDC in obvezno količino $PAYFREIGHT žetonov (kot garancijo in 
//!    pogoj za ugodnejšo provizijo).
//! 2. Nalaganje CMR: Prevoznik opravi prevoz in preko vmesnika naloži hash 
//!    overjenega CMR dokumenta.
//! 3. Izplačilo in Vračilo: Administrator preveri CMR in potrdi izplačilo. 
//!    Pogodba samodejno izplača neto USDC prevozniku, vzame 0.5% provizije 
//!    za platformo ter špediterju vrne zaklenjene $PAYFREIGHT žetone.
//! =============================================================================

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

declare_id!("PayfrE1111111111111111111111111111111111111");

#[program]
pub mod payfreight_escrow {
    use super::*;

    /// 1. Špediter ustvari naročilo ter zaklene USDC voznino in $PAYFREIGHT žetone
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

        // A) Prenos USDC voznine s špediterjevega računa v USDC Vault
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

        // B) Prenos $PAYFREIGHT žetonov s špediterjevega računa v Payfreight Token Vault
        let cpi_accounts_token = Transfer {
            from: ctx.accounts.shipper_payfreight_account.to_account_info(),
            to: ctx.accounts.payfreight_vault.to_account_info(),
            authority: ctx.accounts.shipper.to_account_info(),
        };
        token::transfer(
            CpiContext::new(cpi_program, cpi_accounts_token),
            payfreight_token_amount,
        )?;

        msg!("Escrow ustvarjen: Zaklenjeno {} USDC in {} $PAYFREIGHT.", usdc_amount, payfreight_token_amount);
        Ok(())
    }

    /// 2. Prevoznik naloži potrjen/overjen CMR (URL ali hash dokumenta)
    pub fn upload_cmr(ctx: Context<UploadCMR>, cmr_hash: String) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow_account;

        require!(!escrow.is_completed, EscrowError::AlreadyCompleted);
        require!(
            ctx.accounts.carrier.key() == escrow.carrier,
            EscrowError::UnauthorizedCarrier
        );

        escrow.cmr_hash = cmr_hash;
        escrow.is_cmr_uploaded = true;

        msg!("Overjen CMR uspešno naložen v sistem.");
        Ok(())
    }

    /// 3. Admin ročno preveri CMR ter sprosti USDC prevozniku in vam provizijo, žetone pa vrne špediterju
    pub fn approve_and_release(ctx: Context<ApproveAndRelease>) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow_account;

        require!(!escrow.is_completed, EscrowError::AlreadyCompleted);
        require!(escrow.is_cmr_uploaded, EscrowError::CMRNotUploaded);
        require!(
            ctx.accounts.admin.key() == escrow.admin,
            EscrowError::UnauthorizedAdmin
        );

        // Semena za PDA podpisnika (da pametna pogodba sama sprosti zaklenjena sredstva)
        let order_id_bytes = escrow.order_id.as_bytes();
        let seeds = &[
            b"escrow",
            order_id_bytes,
            &[escrow.bump],
        ];
        let signer_seeds = &[&seeds[..]];

        // Izračun provizije: 0.5% (50 bazičnih točk) platformi, 99.5% prevozniku
        let platform_fee = (escrow.usdc_amount * 50) / 10000;
        let carrier_payout = escrow.usdc_amount - platform_fee;

        let cpi_program = ctx.accounts.token_program.to_account_info();

        // A) Nakazilo neto voznine Prevozniku (USDC)
        let transfer_to_carrier = Transfer {
            from: ctx.accounts.usdc_vault.to_account_info(),
            to: ctx.accounts.carrier_usdc_account.to_account_info(),
            authority: escrow.to_account_info(),
        };
        token::transfer(
            CpiContext::new_with_signer(cpi_program.clone(), transfer_to_carrier, signer_seeds),
            carrier_payout,
        )?;

        // B) Nakazilo provizije na denarnico Payfreight platforme (USDC)
        let transfer_fee = Transfer {
            from: ctx.accounts.usdc_vault.to_account_info(),
            to: ctx.accounts.admin_fee_usdc_account.to_account_info(),
            authority: escrow.to_account_info(),
        };
        token::transfer(
            CpiContext::new_with_signer(cpi_program.clone(), transfer_fee, signer_seeds),
            platform_fee,
        )?;

        // C) Vračilo zaklenjenih $PAYFREIGHT žetonov nazaj Špediterju
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

        msg!("Plačilo sproščeno! Prevoznik: {} USDC, Provizija: {} USDC, Žetoni vrnjeni špediterju.", carrier_payout, platform_fee);
        Ok(())
    }
}

// -----------------------------------------------------------------------------
// STRUKTURE RAČUNOV IN KONTEKSTI
// -----------------------------------------------------------------------------

#[derive(Accounts)]
#[instruction(order_id: String)]
pub struct InitializeEscrow<'info> {
    #[account(mut)]
    pub shipper: Signer<'info>,
    /// CHECK: Prevoznikov javni ključ
    pub carrier: AccountInfo<'info>,
    /// CHECK: Admin javni ključ
    pub admin: AccountInfo<'info>,

    #[account(
        init,
        payer = shipper,
        space = 8 + 32 + 32 + 32 + 8 + 8 + 64 + 128 + 1 + 1 + 1,
        seeds = [b"escrow", order_id.as_bytes()],
        bump
    )]
    pub escrow_account: Account<'info, EscrowAccount>,

    // Sef za USDC voznino
    #[account(
        init,
        payer = shipper,
        seeds = [b"usdc_vault", order_id.as_bytes()],
        bump,
        token::mint = usdc_mint,
        token::authority = escrow_account,
    )]
    pub usdc_vault: Account<'info, TokenAccount>,

    // Sef za $PAYFREIGHT žetone
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
// STANJE ESCROW RAČUNA (STATE)
// -----------------------------------------------------------------------------

#[account]
pub struct EscrowAccount {
    pub shipper: Pubkey,                    // Špediter / Naročnik
    pub carrier: Pubkey,                    // Prevoznik
    pub admin: Pubkey,                      // Administrator
    pub usdc_amount: u64,                   // Znesek voznine v USDC
    pub payfreight_token_amount: u64,     // Količina zaklenjenih $PAYFREIGHT žetonov
    pub order_id: String,                   // ID naročila ali št. fakture
    pub cmr_hash: String,                   // Link/Hash potrjenega CMR-ja
    pub is_cmr_uploaded: bool,            // Ali je CMR naložen
    pub is_completed: bool,                 // Ali je transakcija zaključena
    pub bump: u8,                           // PDA Bump
}

// -----------------------------------------------------------------------------
// NAPAKE (ERRORS)
// -----------------------------------------------------------------------------

#[error_code]
pub enum EscrowError {
    #[msg("Naročilo je že zaključeno.")]
    AlreadyCompleted,
    #[msg("Samo izbrani prevoznik lahko naloži CMR.")]
    UnauthorizedCarrier,
    #[msg("Samo odobreni administrator lahko potrdi izplačilo.")]
    UnauthorizedAdmin,
    #[msg("CMR dokument še ni bil naložen.")]
    CMRNotUploaded,
}
