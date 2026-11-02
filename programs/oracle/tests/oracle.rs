mod common;

use common::{assert_custom_error, setup};
use oracle::{constants::PRICE_DECIMALS, error::OracleError};
use solana_keypair::Keypair;
use solana_signer::Signer;

const ONE_USD: u64 = 1_000_000;

#[test]
fn initialize_sets_admin_decimals_and_slot() {
    let mut env = setup();

    env.initialize().unwrap();

    let state = env.read_oracle();
    assert_eq!(state.admin, env.admin.pubkey());
    assert_eq!(state.decimals, PRICE_DECIMALS);
    assert_eq!(state.price, 0);
    assert_eq!(state.last_updated_slot, env.current_slot());
}

#[test]
fn initialize_twice_fails() {
    let mut env = setup();
    env.initialize().unwrap();
    env.svm.expire_blockhash();

    let res = env.initialize();

    assert!(
        res.is_err(),
        "second initialize must fail: PDA already exists"
    );
}

#[test]
fn update_price_from_admin_succeeds() {
    let mut env = setup();
    env.initialize().unwrap();
    env.svm.warp_to_slot(50);

    let admin = env.admin.insecure_clone();
    env.update_price(&admin, 25 * ONE_USD).unwrap();

    let state = env.read_oracle();
    assert_eq!(state.price, 25 * ONE_USD);
    assert_eq!(state.last_updated_slot, 50);
}

#[test]
fn update_price_from_non_admin_fails() {
    let mut env = setup();
    env.initialize().unwrap();

    let stranger = Keypair::new();
    env.svm
        .airdrop(&stranger.pubkey(), common::ONE_SOL)
        .unwrap();

    let res = env.update_price(&stranger, 25 * ONE_USD);

    assert_custom_error(res, OracleError::Unauthorized.into());
    assert_eq!(env.read_oracle().price, 0, "price must stay untouched");
}

#[test]
fn update_price_with_zero_fails() {
    let mut env = setup();
    env.initialize().unwrap();

    let admin = env.admin.insecure_clone();
    let res = env.update_price(&admin, 0);

    assert_custom_error(res, OracleError::InvalidPrice.into());
}

#[test]
fn update_price_jump_over_20_percent_fails() {
    let mut env = setup();
    env.initialize().unwrap();
    let admin = env.admin.insecure_clone();
    env.update_price(&admin, 100 * ONE_USD).unwrap();

    let too_high = env.update_price(&admin, 121 * ONE_USD);
    assert_custom_error(too_high, OracleError::PriceOutOfRange.into());

    let too_low = env.update_price(&admin, 79 * ONE_USD);
    assert_custom_error(too_low, OracleError::PriceOutOfRange.into());

    env.update_price(&admin, 120 * ONE_USD).unwrap();
    assert_eq!(env.read_oracle().price, 120 * ONE_USD);
}

#[test]
fn require_fresh_distinguishes_fresh_and_stale() {
    let mut env = setup();
    env.initialize().unwrap();
    let admin = env.admin.insecure_clone();
    env.svm.warp_to_slot(100);
    env.update_price(&admin, 25 * ONE_USD).unwrap();

    let state = env.read_oracle();
    assert!(state
        .require_fresh(100 + oracle::constants::MAX_STALENESS_SLOTS)
        .is_ok());
    assert!(state
        .require_fresh(100 + oracle::constants::MAX_STALENESS_SLOTS + 1)
        .is_err());
}
