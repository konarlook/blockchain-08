use std::fmt::Display;
pub fn tx_url(signature: impl Display, rpc_url: &str) -> String {
    format!(
        "https://explorer.solana.com/tx/{signature}{}",
        cluster_query(rpc_url)
    )
}

pub fn address_url(address: impl Display, rpc_url: &str) -> String {
    format!(
        "https://explorer.solana.com/address/{address}{}",
        cluster_query(rpc_url)
    )
}

fn cluster_query(rpc_url: &str) -> String {
    if rpc_url.contains("devnet") {
        "?cluster=devnet".to_owned()
    } else if rpc_url.contains("127.0.0.1") || rpc_url.contains("localhost") {
        format!("?cluster=custom&customUrl={}", url_encode(rpc_url))
    } else {
        String::new()
    }
}

fn url_encode(raw: &str) -> String {
    raw.replace(':', "%3A").replace('/', "%2F")
}
