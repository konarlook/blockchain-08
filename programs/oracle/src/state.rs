use anchor_lang::prelude::*;

use crate::{constants::MAX_STALENESS_SLOTS, error::OracleError};

#[account]
#[derive(InitSpace)]
pub struct OracleState {
    pub admin: Pubkey,
    pub price: u64,
    pub decimals: u8,
    pub last_updated_slot: u64,
    pub bump: u8,
}

impl OracleState {
    pub fn age_in_slots(&self, current_slot: u64) -> u64 {
        current_slot.saturating_sub(self.last_updated_slot)
    }

    pub fn is_fresh(&self, current_slot: u64) -> bool {
        self.age_in_slots(current_slot) <= MAX_STALENESS_SLOTS
    }

    pub fn require_fresh(&self, current_slot: u64) -> Result<()> {
        require!(self.is_fresh(current_slot), OracleError::StaleOracle);
        Ok(())
    }

    pub fn has_reference_price(&self, current_slot: u64) -> bool {
        self.price > 0 && self.is_fresh(current_slot)
    }
}
