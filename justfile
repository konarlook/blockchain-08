set shell := ["bash", "-euo", "pipefail", "-c"]

localnet_rpc := "http://127.0.0.1:8899"
devnet_rpc := "https://api.devnet.solana.com"
treasury_keypair := "keys/treasury.json"

default:
    @just --list

install:
    solana --version
    anchor --version
    cd backend && cargo fetch

validator:
    solana-test-validator --reset --ledger test-ledger

build:
    anchor build

deploy cluster="localnet":
    anchor program deploy --provider.cluster {{cluster}}

treasury:
    mkdir -p keys
    test -f {{ treasury_keypair }} || solana-keygen new --no-bip39-passphrase --silent -o {{ treasury_keypair }}
    @echo "treasury: $(solana address -k {{ treasury_keypair }})"

init cluster="localnet": treasury
    cd backend && RPC_URL={{ if cluster == "devnet" { devnet_rpc } else { localnet_rpc } }} \
        cargo run --release --bin init -- "$(solana address -k ../{{ treasury_keypair }})"

backend cluster="localnet":
    cd backend && RPC_URL={{ if cluster == "devnet" { devnet_rpc } else { localnet_rpc } }} \
        cargo run --release --bin feeder

create-token decimals="6" supply="1000" cluster="localnet":
    cd backend && RPC_URL={{ if cluster == "devnet" { devnet_rpc } else { localnet_rpc } }} \
        cargo run --release --bin create_token -- {{decimals}} {{supply}}

test:
    anchor test
    cd backend && cargo test