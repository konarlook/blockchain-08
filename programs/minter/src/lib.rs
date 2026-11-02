pub mod constants;
pub mod error;

pub mod events;
pub mod fee;
pub mod instructions;
pub mod oracle_guard;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("7Rku9YJHdjyo2PCcHyp8T9jEUrgDZFMxe9djLhync6t");

#[program]
pub mod minter {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, fee_usd: u64) -> Result<()> {
        handle_initialize(ctx, fee_usd)
    }

    pub fn create_token(
        ctx: Context<CreateToken>,
        decimals: u8,
        initial_supply: u64,
    ) -> Result<()> {
        handle_create_token(ctx, decimals, initial_supply)
    }
}
