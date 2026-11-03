use anchor_lang::prelude::*;

use crate::{constants::*, error::OracleError, events::PriceUpdated, state::OracleState};

#[derive(Accounts)]
pub struct UpdatePrice<'info> {
    #[account(
        mut,
        seeds = [ORACLE_SEED],
        bump = oracle.bump,
        has_one = admin @ OracleError::Unauthorized
    )]
    pub oracle: Account<'info, OracleState>,
    pub admin: Signer<'info>,
}

fn check_price_jump(old_price: u64, new_price: u64) -> Result<()> {
    let min = old_price.saturating_mul(MIN_PRICE_CHANGE_NUM) / PRICE_CHANGE_DEN;
    let max = old_price.saturating_mul(MAX_PRICE_CHANGE_NUM) / PRICE_CHANGE_DEN;
    require!(
        (min..=max).contains(&new_price),
        OracleError::PriceOutOfRange
    );
    Ok(())
}

pub fn handle_update_price(ctx: Context<UpdatePrice>, new_price: u64) -> Result<()> {
    require!(new_price > 0, OracleError::InvalidPrice);

    let oracle = &mut ctx.accounts.oracle;
    let current_slot = Clock::get()?.slot;
    if oracle.has_reference_price(current_slot) {
        check_price_jump(oracle.price, new_price)?;
    }

    oracle.price = new_price;
    oracle.last_updated_slot = current_slot;

    emit!(PriceUpdated {
        price: oracle.price,
        slot: oracle.last_updated_slot,
        updater: ctx.accounts.admin.key(),
    });

    Ok(())
}
