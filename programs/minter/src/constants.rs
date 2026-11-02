use anchor_lang::prelude::*;

#[constant]
pub const CONFIG_SEED: &[u8] = b"config";

#[constant]
pub const MINT_AUTHORITY_SEED: &[u8] = b"mint_authority";

#[constant]
pub const LAMPORTS_PER_SOL: u64 = 1_000_000_000;

pub const EXPECTED_ORACLE_DECIMALS: u8 = 6;

pub const MAX_TOKEN_DECIMALS: u8 = 9;
