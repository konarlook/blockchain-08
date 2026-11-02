use anchor_lang::prelude::*;
use oracle::state::OracleState;

use crate::{constants::EXPECTED_ORACLE_DECIMALS, error::MinterError};

pub fn validate_oracle(oracle: &OracleState, current_slot: u64) -> Result<()> {
    require!(
        oracle.decimals == EXPECTED_ORACLE_DECIMALS,
        MinterError::InvalidOracleDecimals,
    );

    require!(oracle.price > 0, MinterError::InvalidOraclePrice);
    oracle.require_fresh(current_slot)
}
