use std::env;

use anchor_client::anchor_lang::prelude::Pubkey;
use anyhow::{Context, Result};
use backend::{
    config::{env_parse_or, Settings},
    connection::{connect, load_keypair, Chain},
    explorer, minter_client, oracle_client,
    retry::with_retries,
};

const DEFAULT_INITIAL_PRICE: u64 = 150_000_000; // 150 USD
const DEFAULT_FEE_USD: u64 = 1_000_000; // 1 USD

fn main() -> Result<()> {
    let settings = Settings::from_env()?;
    let treasury: Pubkey = env::args()
        .nth(1)
        .context("usage: init <TREASURY_PUBKEY>")?
        .parse()?;
    let initial_price = env_parse_or("INITIAL_PRICE", DEFAULT_INITIAL_PRICE)?;
    let fee_usd = env_parse_or("FEE_USD", DEFAULT_FEE_USD)?;
    let chain = connect(&settings, load_keypair(&settings.admin_keypair)?)?;

    let oracle_state = init_oracle(&chain, &settings, initial_price)?;
    let config = init_minter(&chain, &settings, treasury, fee_usd)?;

    println!();
    println!("ORACLE_STATE_PUBKEY={oracle_state}");
    println!("MINTER_CONFIG_PUBKEY={config}");
    println!("TREASURY_PUBKEY={treasury}");
    Ok(())
}

fn init_oracle(chain: &Chain, settings: &Settings, initial_price: u64) -> Result<Pubkey> {
    let state = oracle_client::state_address(&settings.oracle_program_id);
    let existing = oracle_client::read_state(&chain.oracle, state)?;

    if existing.is_none() {
        let sig = with_retries("oracle initialize", || oracle_client::initialize(&chain.oracle, state))?;
        println!("oracle: initialized          {}", explorer::tx_url(sig, &settings.rpc_url));
    }
    if existing.is_none_or(|oracle| oracle.price == 0) {
        let sig = with_retries("oracle first price", || {
            oracle_client::update_price(&chain.oracle, state, initial_price, 0)
        })?;
        println!("oracle: first price {initial_price}  {}", explorer::tx_url(sig, &settings.rpc_url));
    } else {
        println!("oracle: already initialized at {state}");
    }
    Ok(state)
}

fn init_minter(chain: &Chain, settings: &Settings, treasury: Pubkey, fee_usd: u64) -> Result<Pubkey> {
    let config = minter_client::config_address(&settings.minter_program_id);

    if let Some(existing) = minter_client::read_config(&chain.minter, config)? {
        println!("minter: already initialized at {config}, treasury {}", existing.treasury);
        return Ok(config);
    }
    let sig = with_retries("minter initialize", || {
        minter_client::initialize(&chain.minter, config, treasury, fee_usd)
    })?;
    println!("minter: initialized          {}", explorer::tx_url(sig, &settings.rpc_url));
    Ok(config)
}