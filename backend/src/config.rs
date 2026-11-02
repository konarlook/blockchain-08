use std::{env, fmt::Display, path::PathBuf, str::FromStr};

use anchor_client::anchor_lang::prelude::Pubkey;
use anyhow::{anyhow, Context, Result};

pub const DEFAULT_RPC_URL: &str = "http://127.0.0.1:8899";
pub const DEFAULT_ADMIN_KEYPAIR: &str = "~/.config/solana/id.json";

pub struct Settings {
    pub rpc_url: String,
    pub admin_keypair: PathBuf,
    pub oracle_program_id: Pubkey,
    pub minter_program_id: Pubkey,
}

impl Settings {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();
        Ok(Self {
            rpc_url: env_or("RPC_URL", DEFAULT_RPC_URL),
            admin_keypair: expand_home(&env_or("ADMIN_KEYPAIR", DEFAULT_ADMIN_KEYPAIR)),
            oracle_program_id: env_parse_or("ORACLE_PROGRAM_ID", oracle::ID)?,
            minter_program_id: env_parse_or("MINTER_PROGRAM_ID", minter::ID)?,
        })
    }
}

pub fn env_or(name: &str, default: &str) -> String {
    env::var(name)
        .ok()
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| default.to_owned())
}

pub fn env_parse_or<T>(name: &str, default: T) -> Result<T>
where
    T: FromStr,
    T::Err: Display,
{
    match env::var(name).ok().filter(|value| !value.is_empty()) {
        Some(raw) => raw.parse().map_err(|err| anyhow!("{name}={raw}: {err}")),
        None => Ok(default),
    }
}

pub fn env_required<T>(name: &str) -> Result<T>
where
    T: FromStr,
    T::Err: Display,
{
    let raw =
        env::var(name).with_context(|| format!("{name} is not set (see backend/.env.example)"))?;
    raw.parse().map_err(|err| anyhow!("{name}={raw}: {err}"))
}

fn expand_home(path: &str) -> PathBuf {
    match path.strip_prefix("~/") {
        Some(rest) => PathBuf::from(env::var("HOME").unwrap_or_default()).join(rest),
        None => PathBuf::from(path),
    }
}
