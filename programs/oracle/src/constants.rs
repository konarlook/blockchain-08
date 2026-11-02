use anchor_lang::prelude::*;

#[constant]
pub const ORACLE_SEED: &[u8] = b"oracle";

#[constant]
pub const PRICE_DECIMALS: u8 = 6;

#[constant]
pub const MAX_STALENESS_SLOTS: u64 = 100;

pub const MAX_PRICE_CHANGE_NUM: u64 = 12;
pub const MIN_PRICE_CHANGE_NUM: u64 = 8;
pub const PRICE_CHANGE_DEN: u64 = 10;
