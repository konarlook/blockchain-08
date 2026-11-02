use anchor_lang::prelude::*;

use crate::{constants::*, state::OracleState};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = admin,
        space = 8 + OracleState::INIT_SPACE,
        seeds = [ORACLE_SEED],
        bump
    )]
    pub oracle: Account<'info, OracleState>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handle_initialize(ctx: Context<Initialize>) -> Result<()> {
    let oracle = &mut ctx.accounts.oracle;

    oracle.admin = ctx.accounts.admin.key();
    oracle.price = 0;
    oracle.decimals = PRICE_DECIMALS;
    oracle.last_updated_slot = Clock::get()?.slot;
    oracle.bump = ctx.bumps.oracle;

    Ok(())
}
