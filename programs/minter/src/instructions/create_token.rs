use anchor_lang::{prelude::*, system_program};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, MintTo, Token, TokenAccount},
};
use oracle::{constants::ORACLE_SEED, state::OracleState};

use crate::{
    constants::*, error::MinterError, events::TokenCreated, fee::calc_fee_lamports,
    oracle_guard::validate_oracle, state::MinterConfig,
};

#[derive(Accounts)]
#[instruction(decimals: u8)]
pub struct CreateToken<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        seeds = [CONFIG_SEED],
        bump = config.bump,
        has_one = treasury @ MinterError::InvalidTreasury
    )]
    pub config: Account<'info, MinterConfig>,

    #[account(
        seeds = [ORACLE_SEED],
        bump = oracle.bump,
        seeds::program = oracle::ID
    )]
    pub oracle: Account<'info, OracleState>,

    #[account(mut)]
    pub treasury: SystemAccount<'info>,

    #[account(
        init,
        payer = payer,
        mint::decimals = decimals,
        mint::authority = mint_authority
    )]
    pub mint: Account<'info, Mint>,

    /// CHECK: PDA без данных - только подписывает mint_to от имени программы.
    /// Адрес проверен seeds-ограничениями
    #[account(seeds = [MINT_AUTHORITY_SEED, mint.key().as_ref()], bump)]
    pub mint_authority: UncheckedAccount<'info>,

    #[account(
        init,
        payer = payer,
        associated_token::mint = mint,
        associated_token::authority = payer
    )]
    pub payer_ata: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handle_create_token(
    ctx: Context<CreateToken>,
    decimals: u8,
    initial_supply: u64,
) -> Result<()> {
    require!(
        decimals <= MAX_TOKEN_DECIMALS,
        MinterError::InvalidTokenDecimals
    );
    require!(initial_supply > 0, MinterError::InvalidSupply);

    let accounts = &ctx.accounts;
    validate_oracle(&accounts.oracle, Clock::get()?.slot)?;

    let fee_lamports = calc_fee_lamports(accounts.config.fee_usd, accounts.oracle.price)?;
    accounts.pay_fee(fee_lamports)?;
    accounts.mint_initial_supply(ctx.bumps.mint_authority, initial_supply)?;

    emit!(TokenCreated {
        mint: accounts.mint.key(),
        creator: accounts.payer.key(),
        decimals,
        initial_supply,
        fee_lamports,
        oracle_price: accounts.oracle.price,
        oracle_slot: accounts.oracle.last_updated_slot,
    });
    Ok(())
}

impl<'info> CreateToken<'info> {
    fn pay_fee(&self, lamports: u64) -> Result<()> {
        let cpi_ctx = CpiContext::new(
            self.system_program.key(),
            system_program::Transfer {
                from: self.payer.to_account_info(),
                to: self.treasury.to_account_info(),
            },
        );
        system_program::transfer(cpi_ctx, lamports)
    }

    fn mint_initial_supply(&self, authority_bump: u8, amount: u64) -> Result<()> {
        let mint_key = self.mint.key();
        let seeds: &[&[u8]] = &[MINT_AUTHORITY_SEED, mint_key.as_ref(), &[authority_bump]];
        let signer_seeds = &[seeds];

        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.key(),
            MintTo {
                mint: self.mint.to_account_info(),
                to: self.payer_ata.to_account_info(),
                authority: self.mint_authority.to_account_info(),
            },
            signer_seeds,
        );
        token::mint_to(cpi_ctx, amount)
    }
}
