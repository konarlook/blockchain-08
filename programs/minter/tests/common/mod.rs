use anchor_lang::{
    prelude::Pubkey,
    solana_program::{
        instruction::{error::InstructionError, Instruction},
        system_program,
    },
    AccountDeserialize, Event, InstructionData, ToAccountMetas,
};
use anchor_spl::{
    associated_token::{self, get_associated_token_address},
    token::{self, Mint, TokenAccount},
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use litesvm::{types::TransactionResult, LiteSVM};
use solana_keypair::Keypair;
use solana_message::{Message, VersionedMessage};
use solana_signer::Signer;
use solana_transaction::versioned::VersionedTransaction;
use solana_transaction_error::TransactionError;

pub const ONE_SOL: u64 = 1_000_000_000;
pub const ONE_USD: u64 = 1_000_000;

pub struct TestEnv {
    pub svm: LiteSVM,
    pub admin: Keypair,
    pub treasury: Keypair,
    pub user: Keypair,
    pub oracle_pda: Pubkey,
    pub config_pda: Pubkey,
}

pub fn setup() -> TestEnv {
    let mut svm = LiteSVM::new();
    let oracle_so = include_bytes!(concat!(env!("CARGO_TARGET_TMPDIR"), "/../deploy/oracle.so"));
    let minter_so = include_bytes!(concat!(env!("CARGO_TARGET_TMPDIR"), "/../deploy/minter.so"));
    svm.add_program(oracle::id(), oracle_so).unwrap();
    svm.add_program(minter::id(), minter_so).unwrap();

    let admin = Keypair::new();
    let treasury = Keypair::new();
    let user = Keypair::new();
    for who in [&admin, &treasury, &user] {
        svm.airdrop(&who.pubkey(), 10 * ONE_SOL).unwrap();
    }

    let (oracle_pda, _) = Pubkey::find_program_address(&[oracle::ORACLE_SEED], &oracle::id());
    let (config_pda, _) = Pubkey::find_program_address(&[minter::CONFIG_SEED], &minter::id());

    TestEnv {
        svm,
        admin,
        treasury,
        user,
        oracle_pda,
        config_pda,
    }
}

impl TestEnv {
    pub fn send(
        &mut self,
        ix: Instruction,
        payer: &Keypair,
        extra_signers: &[&Keypair],
    ) -> TransactionResult {
        self.svm.expire_blockhash();
        let blockhash = self.svm.latest_blockhash();
        let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);

        let mut signers: Vec<&Keypair> = vec![payer];
        signers.extend_from_slice(extra_signers);

        let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &signers).unwrap();
        self.svm.send_transaction(tx)
    }

    pub fn init_oracle_with_price(&mut self, price: u64) {
        let admin = self.admin.insecure_clone();

        let init = Instruction::new_with_bytes(
            oracle::id(),
            &oracle::instruction::Initialize {}.data(),
            oracle::accounts::Initialize {
                oracle: self.oracle_pda,
                admin: admin.pubkey(),
                system_program: system_program::ID,
            }
            .to_account_metas(None),
        );
        self.send(init, &admin, &[]).unwrap();

        if price > 0 {
            let update = Instruction::new_with_bytes(
                oracle::id(),
                &oracle::instruction::UpdatePrice { new_price: price }.data(),
                oracle::accounts::UpdatePrice {
                    oracle: self.oracle_pda,
                    admin: admin.pubkey(),
                }
                .to_account_metas(None),
            );
            self.send(update, &admin, &[]).unwrap();
        }
    }

    pub fn init_minter(&mut self, fee_usd: u64) {
        let admin = self.admin.insecure_clone();
        let ix = Instruction::new_with_bytes(
            minter::id(),
            &minter::instruction::Initialize { fee_usd }.data(),
            minter::accounts::Initialize {
                config: self.config_pda,
                admin: admin.pubkey(),
                treasury: self.treasury.pubkey(),
                system_program: system_program::ID,
            }
            .to_account_metas(None),
        );
        self.send(ix, &admin, &[]).unwrap();
    }

    pub fn create_token_with(
        &mut self,
        mint: &Keypair,
        decimals: u8,
        initial_supply: u64,
        oracle: Pubkey,
        treasury: Pubkey,
    ) -> TransactionResult {
        let user = self.user.insecure_clone();
        let (mint_authority, _) = Pubkey::find_program_address(
            &[minter::MINT_AUTHORITY_SEED, mint.pubkey().as_ref()],
            &minter::id(),
        );

        let ix = Instruction::new_with_bytes(
            minter::id(),
            &minter::instruction::CreateToken {
                decimals,
                initial_supply,
            }
            .data(),
            minter::accounts::CreateToken {
                payer: user.pubkey(),
                config: self.config_pda,
                oracle,
                treasury,
                mint: mint.pubkey(),
                mint_authority,
                payer_ata: get_associated_token_address(&user.pubkey(), &mint.pubkey()),
                token_program: token::ID,
                associated_token_program: associated_token::ID,
                system_program: system_program::ID,
            }
            .to_account_metas(None),
        );
        self.send(ix, &user, &[mint])
    }

    pub fn create_token(
        &mut self,
        mint: &Keypair,
        decimals: u8,
        initial_supply: u64,
    ) -> TransactionResult {
        let (oracle, treasury) = (self.oracle_pda, self.treasury.pubkey());
        self.create_token_with(mint, decimals, initial_supply, oracle, treasury)
    }

    pub fn read_mint(&self, mint: &Pubkey) -> Mint {
        let account = self.svm.get_account(mint).expect("mint must exist");
        Mint::try_deserialize(&mut account.data.as_slice()).unwrap()
    }

    pub fn read_token_account(&self, address: &Pubkey) -> TokenAccount {
        let account = self
            .svm
            .get_account(address)
            .expect("token account must exist");
        TokenAccount::try_deserialize(&mut account.data.as_slice()).unwrap()
    }

    pub fn balance(&self, address: &Pubkey) -> u64 {
        self.svm.get_balance(address).unwrap_or(0)
    }

    pub fn oracle_last_updated_slot(&self) -> u64 {
        let account = self.svm.get_account(&self.oracle_pda).unwrap();
        oracle::OracleState::try_deserialize(&mut account.data.as_slice())
            .unwrap()
            .last_updated_slot
    }
}

pub fn find_event<E: Event>(logs: &[String]) -> Option<E> {
    logs.iter()
        .filter_map(|line| line.strip_prefix("Program data: "))
        .filter_map(|encoded| BASE64.decode(encoded).ok())
        .find(|bytes| bytes.starts_with(E::DISCRIMINATOR))
        .map(|bytes| E::try_from_slice(&bytes[E::DISCRIMINATOR.len()..]).unwrap())
}

pub fn assert_custom_error(res: TransactionResult, expected: u32) {
    let failed = res.expect_err("transaction was expected to fail");
    match failed.err {
        TransactionError::InstructionError(_, InstructionError::Custom(code)) => {
            assert_eq!(
                code, expected,
                "unexpected error code; logs: {:#?}",
                failed.meta.logs
            )
        }
        other => panic!(
            "unexpected error kind: {other:?}; logs: {:#?}",
            failed.meta.logs
        ),
    }
}
