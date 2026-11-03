use anchor_lang::{prelude::*, solana_program::bpf_loader_upgradeable};

use crate::{constants::*, error::OracleError, state::OracleState};

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
    #[account(
        seeds = [crate::ID.as_ref()],
        bump,
        seeds::program = bpf_loader_upgradeable::ID,
        constraint = program_data.upgrade_authority_address == Some(admin.key())
            @ OracleError::Unauthorized
    )]
    pub program_data: Account<'info, ProgramData>,
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
