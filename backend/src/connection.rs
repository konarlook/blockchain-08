use std::{path::Path, rc::Rc, str::FromStr};

use anchor_client::{Client, Cluster, CommitmentConfig, Program};
use anyhow::{anyhow, Result};
use solana_keypair::{read_keypair_file, Keypair};

use crate::config::Settings;

pub type Payer = Rc<Keypair>;


pub struct Chain {
    pub payer: Payer,
    pub oracle: Program<Payer>,
    pub minter: Program<Payer>,
}

pub fn load_keypair(path: &Path) -> Result<Keypair> {
    read_keypair_file(path).map_err(|err| anyhow!("cannot read keypair {}: {err}", path.display()))
}

pub fn connect(settings: &Settings, payer: Keypair) -> Result<Chain> {
    let payer = Rc::new(payer);
    let cluster = Cluster::from_str(&settings.rpc_url)?;
    let client = Client::new_with_options(cluster, payer.clone(), CommitmentConfig::confirmed());

    Ok(Chain {
        oracle: client.program(settings.oracle_program_id)?,
        minter: client.program(settings.minter_program_id)?,
        payer,
    })
}