use std::env;

use anyhow::{Context, Result};
use backend::{
    config::Settings,
    connection::{connect, load_keypair},
    explorer, minter_client, oracle_client,
    retry::send_with_retries,
};
use solana_keypair::Keypair;

fn main() -> Result<()> {
    let settings = Settings::from_env()?;
    let (decimals, initial_supply) = parse_args()?;
    let chain = connect(&settings, load_keypair(&settings.admin_keypair)?)?;

    let config_address = minter_client::config_address(&settings.minter_program_id);
    let config = minter_client::read_config(&chain.minter, config_address)?
        .context("minter is not initialized: run `just init` first")?;
    let oracle_state = oracle_client::state_address(&settings.oracle_program_id);

    let mint = Keypair::new();
    let accounts = minter_client::create_token_accounts(&chain.minter, &mint);
    let tx = minter_client::create_token_tx(
        &chain.minter,
        config_address,
        oracle_state,
        config.treasury,
        &mint,
        decimals,
        initial_supply,
    )?;
    let signature = send_with_retries("create_token", &chain.minter, &tx)?;

    println!(
        "transaction: {}",
        explorer::tx_url(signature, &settings.rpc_url)
    );
    println!(
        "mint:        {}",
        explorer::address_url(accounts.mint, &settings.rpc_url)
    );
    println!("your ATA:    {}", accounts.creator_ata);
    println!("supply:      {initial_supply} (decimals {decimals})");
    Ok(())
}

fn parse_args() -> Result<(u8, u64)> {
    let mut args = env::args().skip(1);
    let usage = "usage: create_token <DECIMALS> <SUPPLY_IN_WHOLE_TOKENS>";
    let decimals: u8 = args.next().context(usage)?.parse()?;
    let whole_tokens: u64 = args.next().context(usage)?.parse()?;

    let initial_supply = 10u64
        .checked_pow(u32::from(decimals))
        .and_then(|scale| whole_tokens.checked_mul(scale))
        .context("supply * 10^decimals does not fit into u64")?;
    Ok((decimals, initial_supply))
}
