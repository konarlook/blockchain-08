use anchor_lang::prelude::*;

use crate::{constants::CONFIG_SEED, state::MinterConfig};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = admin,
        space = 8 + MinterConfig::INIT_SPACE,
        seeds = [CONFIG_SEED],
        bump
    )]
    pub config: Account<'info, MinterConfig>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub treasury: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handle_initialize(ctx: Context<Initialize>, fee_usd: u64) -> Result<()> {
    let config = &mut ctx.accounts.config;
    config.admin = ctx.accounts.admin.key();
    config.treasury = ctx.accounts.treasury.key();
    config.fee_usd = fee_usd;
    config.bump = ctx.bumps.config;
    Ok(())
}
