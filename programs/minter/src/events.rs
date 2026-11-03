use anchor_lang::prelude::*;

#[event]
pub struct TokenCreated {
    pub mint: Pubkey,
    pub creator: Pubkey,
    pub decimals: u8,
    pub initial_supply: u64,
    pub fee_lamports: u64,
    pub oracle_price: u64,
    pub oracle_slot: u64,
}

#[event]
pub struct ConfigUpdated {
    pub fee_usd: u64,
    pub treasury: Pubkey,
}
