use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct MinterConfig {
    pub admin: Pubkey,
    pub treasury: Pubkey,
    pub fee_usd: u64,
    pub bump: u8,
}
