use anchor_lang::prelude::*;
use oracle::{constants::PRICE_DECIMALS, state::OracleState};

use crate::error::MinterError;

pub fn validate_oracle(oracle: &OracleState, current_slot: u64) -> Result<()> {
    require!(
        oracle.decimals == PRICE_DECIMALS,
        MinterError::InvalidOracleDecimals,
    );

    require!(oracle.price > 0, MinterError::InvalidOraclePrice);
    oracle.require_fresh(current_slot)
}
