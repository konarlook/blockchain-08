use anchor_lang::prelude::*;

#[event]
pub struct PriceUpdated {
    pub price: u64,
    pub slot: u64,
    pub updater: Pubkey,
}
