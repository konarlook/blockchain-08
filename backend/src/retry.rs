use std::{thread, time::Duration};

use anchor_client::ClientError;

pub const MAX_ATTEMPTS: u32 = 3;
const BASE_BACKOFF: Duration = Duration::from_millis(500);

pub fn with_retries<T>(
    label: &str,
    mut operation: impl FnMut() -> Result<T, ClientError>,
) -> Result<T, ClientError> {
    let mut attempt = 1;
    loop {
        match operation() {
            Ok(value) => return Ok(value),
            Err(err) if attempt < MAX_ATTEMPTS && is_transient(&err) => {
                let backoff = BASE_BACKOFF * 2u32.pow(attempt - 1);
                eprintln!("[{label}] attempt {attempt}/{MAX_ATTEMPTS} failed: {err}; retry in {backoff:?}");
                thread::sleep(backoff);
                attempt += 1;
            }
            Err(err) => return Err(err),
        }
    }
}

fn is_transient(err: &ClientError) -> bool {
    match err {
        ClientError::SolanaClientError(rpc_err) => rpc_err.get_transaction_error().is_none(),
        _ => false,
    }
}
