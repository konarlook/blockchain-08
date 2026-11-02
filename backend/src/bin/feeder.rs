use std::{
    thread,
    time::{Duration, Instant},
};

use anchor_client::anchor_lang::prelude::Pubkey;
use anyhow::{bail, Result};
use backend::{
    config::{env_parse_or, env_required, Settings},
    connection::{connect, load_keypair, Chain},
    explorer, oracle_client,
    price_source::{MockPriceSource, PriceSource},
    publish_policy::{Decision, PublishPolicy},
    retry::with_retries,
};

const DEFAULT_INITIAL_PRICE: u64 = 150_000_000;

struct Feeder {
    chain: Chain,
    rpc_url: String,
    oracle_state: Pubkey,
    priority_fee: u64,
    source: MockPriceSource,
    policy: PublishPolicy,
}

fn main() -> Result<()> {
    let settings = Settings::from_env()?;
    let oracle_state: Pubkey = env_required("ORACLE_STATE_PUBKEY")?;
    ensure_is_oracle_pda(&settings, oracle_state)?;

    let poll = secs("FEEDER_POLL_SECS", 2)?;
    let policy = PublishPolicy::new(
        secs("FEEDER_MIN_INTERVAL_SECS", 10)?,
        secs("FEEDER_HEARTBEAT_SECS", 30)?,
    );
    let chain = connect(&settings, load_keypair(&settings.admin_keypair)?)?;
    let start_price = start_price(&chain, oracle_state)?;

    let mut feeder = Feeder {
        chain,
        rpc_url: settings.rpc_url,
        oracle_state,
        priority_fee: env_parse_or("PRIORITY_FEE_MICRO_LAMPORTS", 0)?,
        source: MockPriceSource::new(start_price),
        policy,
    };
    println!("feeder: oracle {oracle_state}, start price {start_price}, poll every {poll:?}");

    loop {
        if let Err(err) = feeder.tick() {
            eprintln!("feeder: {err:#}");
        }
        thread::sleep(poll);
    }
}

impl Feeder {
    fn tick(&mut self) -> Result<()> {
        let price = self.source.next_price()?;
        let now = Instant::now();

        match self.policy.decide(price, now) {
            Decision::Publish => {
                self.publish(price)?;
                self.policy.record_sent(price, now);
            }
            Decision::SkipSameValue => println!("feeder: price {price} unchanged, skip"),
            Decision::SkipTooSoon => {}
        }
        Ok(())
    }

    fn publish(&self, price: u64) -> Result<()> {
        let signature = with_retries("update_price", || {
            oracle_client::update_price(
                &self.chain.oracle,
                self.oracle_state,
                price,
                self.priority_fee,
            )
        })?;
        let state = oracle_client::read_state(&self.chain.oracle, self.oracle_state)?;
        let slot = state.map_or(0, |oracle| oracle.last_updated_slot);
        println!(
            "feeder: price {price} -> slot {slot}  {}",
            explorer::tx_url(signature, &self.rpc_url)
        );
        Ok(())
    }
}

fn ensure_is_oracle_pda(settings: &Settings, oracle_state: Pubkey) -> Result<()> {
    let expected = oracle_client::state_address(&settings.oracle_program_id);
    if oracle_state != expected {
        bail!("ORACLE_STATE_PUBKEY={oracle_state}, but oracle PDA for this program is {expected}");
    }
    Ok(())
}

fn start_price(chain: &Chain, oracle_state: Pubkey) -> Result<u64> {
    let on_chain =
        oracle_client::read_state(&chain.oracle, oracle_state)?.map_or(0, |oracle| oracle.price);
    if on_chain > 0 {
        return Ok(on_chain);
    }
    env_parse_or("INITIAL_PRICE", DEFAULT_INITIAL_PRICE)
}

fn secs(name: &str, default: u64) -> Result<Duration> {
    Ok(Duration::from_secs(env_parse_or(name, default)?))
}
