use anchor_lang::prelude::*;

use crate::{
    constants::CONFIG_SEED, error::MinterError, events::ConfigUpdated, state::MinterConfig,
};

#[derive(Accounts)]
pub struct UpdateConfig<'info> {
    #[account(
        mut,
        seeds = [CONFIG_SEED],
        bump = config.bump,
        has_one = admin @ MinterError::Unauthorized
    )]
    pub config: Account<'info, MinterConfig>,
    pub admin: Signer<'info>,
    pub treasury: SystemAccount<'info>,
}

pub fn handle_update_config(ctx: Context<UpdateConfig>, fee_usd: u64) -> Result<()> {
    let config = &mut ctx.accounts.config;
    config.fee_usd = fee_usd;
    config.treasury = ctx.accounts.treasury.key();

    emit!(ConfigUpdated {
        fee_usd: config.fee_usd,
        treasury: config.treasury,
    });
    Ok(())
}
