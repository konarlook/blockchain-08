mod common;

use anchor_lang::{
    error::ErrorCode as AnchorErrorCode, prelude::Pubkey, AccountDeserialize, AccountSerialize,
};
use anchor_spl::associated_token::get_associated_token_address;
use common::{assert_custom_error, find_event, setup, ONE_SOL, ONE_USD};
use minter::{
    error::MinterError,
    events::{ConfigUpdated, TokenCreated},
};
use oracle::{constants::MAX_STALENESS_SLOTS, error::OracleError, OracleState};
use solana_account::Account;
use solana_keypair::Keypair;
use solana_signer::Signer;

const PRICE: u64 = 25 * ONE_USD;
const FEE_USD: u64 = ONE_USD;
const EXPECTED_FEE_LAMPORTS: u64 = 40_000_000;
const DECIMALS: u8 = 6;
const SUPPLY: u64 = 1_000 * 1_000_000;

#[test]
fn create_token_charges_fee_and_mints_supply() {
    let mut env = setup();
    env.init_oracle_with_price(PRICE);
    env.init_minter(FEE_USD);
    let mint = Keypair::new();
    let treasury_before = env.balance(&env.treasury.pubkey());

    let meta = env.create_token(&mint, DECIMALS, SUPPLY).unwrap();

    let treasury_after = env.balance(&env.treasury.pubkey());
    assert_eq!(treasury_after - treasury_before, EXPECTED_FEE_LAMPORTS);

    let mint_state = env.read_mint(&mint.pubkey());
    assert_eq!(mint_state.decimals, DECIMALS);
    assert_eq!(mint_state.supply, SUPPLY);
    let (expected_authority, _) = Pubkey::find_program_address(
        &[minter::MINT_AUTHORITY_SEED, mint.pubkey().as_ref()],
        &minter::id(),
    );
    assert_eq!(mint_state.mint_authority, Some(expected_authority).into());

    let ata = get_associated_token_address(&env.user.pubkey(), &mint.pubkey());
    let ata_state = env.read_token_account(&ata);
    assert_eq!(ata_state.amount, SUPPLY);
    assert_eq!(ata_state.owner, env.user.pubkey());
    assert_eq!(ata_state.mint, mint.pubkey());

    let event: TokenCreated = find_event(&meta.logs).expect("TokenCreated must be emitted");
    assert_eq!(event.mint, mint.pubkey());
    assert_eq!(event.creator, env.user.pubkey());
    assert_eq!(event.fee_lamports, EXPECTED_FEE_LAMPORTS);
    assert_eq!(event.oracle_price, PRICE);
}

#[test]
fn create_token_with_stale_oracle_fails() {
    let mut env = setup();
    env.init_oracle_with_price(PRICE);
    env.init_minter(FEE_USD);
    let stale_slot = env.oracle_last_updated_slot() + MAX_STALENESS_SLOTS + 1;
    env.svm.warp_to_slot(stale_slot);

    let res = env.create_token(&Keypair::new(), DECIMALS, SUPPLY);

    assert_custom_error(res, OracleError::StaleOracle.into());
}

#[test]
fn create_token_with_oracle_on_the_staleness_edge_succeeds() {
    let mut env = setup();
    env.init_oracle_with_price(PRICE);
    env.init_minter(FEE_USD);
    let edge_slot = env.oracle_last_updated_slot() + MAX_STALENESS_SLOTS;
    env.svm.warp_to_slot(edge_slot);

    env.create_token(&Keypair::new(), DECIMALS, SUPPLY).unwrap();
}

#[test]
fn create_token_before_first_price_fails() {
    let mut env = setup();
    env.init_oracle_with_price(0);
    env.init_minter(FEE_USD);

    let res = env.create_token(&Keypair::new(), DECIMALS, SUPPLY);

    assert_custom_error(res, MinterError::InvalidOraclePrice.into());
}

#[test]
fn create_token_with_wrong_oracle_decimals_fails() {
    let mut env = setup();
    env.init_oracle_with_price(PRICE);
    env.init_minter(FEE_USD);

    let mut account = env.svm.get_account(&env.oracle_pda).unwrap();
    let mut state = deserialize_oracle(&account.data);
    state.decimals = 8;
    account.data = serialize_oracle(&state);
    env.svm.set_account(env.oracle_pda, account).unwrap();

    let res = env.create_token(&Keypair::new(), DECIMALS, SUPPLY);

    assert_custom_error(res, MinterError::InvalidOracleDecimals.into());
}

#[test]
fn create_token_with_spoofed_oracle_fails() {
    let mut env = setup();
    env.init_oracle_with_price(PRICE);
    env.init_minter(FEE_USD);

    let fake_oracle = Pubkey::new_unique();
    let fake_state = OracleState {
        admin: env.user.pubkey(),
        price: 1_000_000 * ONE_USD,
        decimals: 6,
        last_updated_slot: env.oracle_last_updated_slot(),
        bump: 255,
    };
    let fake_account = Account {
        lamports: ONE_SOL,
        data: serialize_oracle(&fake_state),
        owner: Pubkey::new_unique(),
        executable: false,
        rent_epoch: 0,
    };
    env.svm.set_account(fake_oracle, fake_account).unwrap();
    let treasury = env.treasury.pubkey();

    let res = env.create_token_with(&Keypair::new(), DECIMALS, SUPPLY, fake_oracle, treasury);

    assert_custom_error(res, AnchorErrorCode::AccountOwnedByWrongProgram.into());
}

#[test]
fn create_token_with_foreign_treasury_fails() {
    let mut env = setup();
    env.init_oracle_with_price(PRICE);
    env.init_minter(FEE_USD);
    let attacker_wallet = Keypair::new().pubkey();
    let oracle = env.oracle_pda;

    let res = env.create_token_with(&Keypair::new(), DECIMALS, SUPPLY, oracle, attacker_wallet);

    assert_custom_error(res, MinterError::InvalidTreasury.into());
}

#[test]
fn create_token_with_zero_supply_fails() {
    let mut env = setup();
    env.init_oracle_with_price(PRICE);
    env.init_minter(FEE_USD);

    let res = env.create_token(&Keypair::new(), DECIMALS, 0);

    assert_custom_error(res, MinterError::InvalidSupply.into());
}

#[test]
fn initialize_by_non_upgrade_authority_fails() {
    let mut env = setup();
    let attacker = env.user.insecure_clone();

    let res = env.init_minter_by(&attacker, FEE_USD);

    assert_custom_error(res, MinterError::Unauthorized.into());
    assert!(env.svm.get_account(&env.config_pda).is_none());
}

#[test]
fn update_config_by_admin_changes_fee_and_treasury() {
    let mut env = setup();
    env.init_oracle_with_price(PRICE);
    env.init_minter(FEE_USD);
    let admin = env.admin.insecure_clone();
    let new_treasury = Keypair::new();
    env.svm.airdrop(&new_treasury.pubkey(), ONE_SOL).unwrap();

    let meta = env
        .update_config(&admin, 2 * FEE_USD, new_treasury.pubkey())
        .unwrap();

    let config = env.read_config();
    assert_eq!(config.fee_usd, 2 * FEE_USD);
    assert_eq!(config.treasury, new_treasury.pubkey());
    let event: ConfigUpdated = find_event(&meta.logs).expect("ConfigUpdated must be emitted");
    assert_eq!(event.fee_usd, 2 * FEE_USD);

    let before = env.balance(&new_treasury.pubkey());
    let (oracle, treasury) = (env.oracle_pda, new_treasury.pubkey());
    env.create_token_with(&Keypair::new(), DECIMALS, SUPPLY, oracle, treasury)
        .unwrap();
    assert_eq!(
        env.balance(&new_treasury.pubkey()) - before,
        2 * EXPECTED_FEE_LAMPORTS
    );
}

#[test]
fn update_config_by_non_admin_fails() {
    let mut env = setup();
    env.init_oracle_with_price(PRICE);
    env.init_minter(FEE_USD);
    let attacker = env.user.insecure_clone();

    let res = env.update_config(&attacker, 0, attacker.pubkey());

    assert_custom_error(res, MinterError::Unauthorized.into());
    assert_eq!(env.read_config().fee_usd, FEE_USD);
}

fn serialize_oracle(state: &OracleState) -> Vec<u8> {
    let mut data = Vec::new();
    state.try_serialize(&mut data).unwrap();
    data
}

fn deserialize_oracle(mut data: &[u8]) -> OracleState {
    OracleState::try_deserialize(&mut data).unwrap()
}
