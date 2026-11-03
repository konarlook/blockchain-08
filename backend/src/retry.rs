use std::{thread, time::Duration};

use anchor_client::{CommitmentConfig, Program, Transaction};
use anyhow::{anyhow, Context, Result};
use solana_signature::Signature;

use crate::connection::Payer;

pub const MAX_ATTEMPTS: u32 = 3;
const BASE_BACKOFF: Duration = Duration::from_millis(500);

pub fn send_with_retries(
    label: &str,
    program: &Program<Payer>,
    tx: &Transaction,
) -> Result<Signature> {
    let rpc = program.rpc();
    let signature = tx.signatures[0];
    let mut attempt = 1;
    loop {
        let err = match rpc.send_and_confirm_transaction(tx) {
            Ok(_) => return Ok(signature),
            Err(err) => err,
        };

        let status =
            rpc.get_signature_status_with_commitment(&signature, CommitmentConfig::processed());
        if let Ok(Some(result)) = status {
            result
                .map_err(|tx_err| anyhow!("[{label}] transaction {signature} failed: {tx_err}"))?;
            rpc.poll_for_signature(&signature)
                .with_context(|| format!("[{label}] waiting for confirmation of {signature}"))?;
            eprintln!("[{label}] RPC returned an error ({err}), but {signature} is confirmed");
            return Ok(signature);
        }

        if attempt >= MAX_ATTEMPTS || err.get_transaction_error().is_some() {
            return Err(anyhow!("[{label}] attempt {attempt}/{MAX_ATTEMPTS}: {err}"));
        }
        let backoff = BASE_BACKOFF * 2u32.pow(attempt - 1);
        eprintln!("[{label}] attempt {attempt}/{MAX_ATTEMPTS} failed: {err}; retry in {backoff:?}");
        thread::sleep(backoff);
        attempt += 1;
    }
}
