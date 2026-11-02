use anchor_lang::{
    prelude::Pubkey,
    solana_program::{
        instruction::{error::InstructionError, Instruction},
        system_program,
    },
    AccountDeserialize, InstructionData, ToAccountMetas,
};
use litesvm::{types::TransactionResult, LiteSVM};
use oracle::{constants::ORACLE_SEED, state::OracleState};
use solana_keypair::Keypair;
use solana_message::{Message, VersionedMessage};
use solana_signer::Signer;
use solana_transaction::versioned::VersionedTransaction;
use solana_transaction_error::TransactionError;

pub const ONE_SOL: u64 = 1_000_000_000;

pub struct TestEnv {
    pub svm: LiteSVM,
    pub admin: Keypair,
    pub oracle_pda: Pubkey,
}

pub fn setup() -> TestEnv {
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!(concat!(env!("CARGO_TARGET_TMPDIR"), "/../deploy/oracle.so"));
    svm.add_program(oracle::id(), bytes).unwrap();

    let admin = Keypair::new();
    svm.airdrop(&admin.pubkey(), 10 * ONE_SOL).unwrap();
    let (oracle_pda, _bump) = Pubkey::find_program_address(&[ORACLE_SEED], &oracle::id());

    TestEnv {
        svm,
        admin,
        oracle_pda,
    }
}

impl TestEnv {
    pub fn send(
        &mut self,
        ix: Instruction,
        payer: &Keypair,
        extra_signers: &[&Keypair],
    ) -> TransactionResult {
        let blockhash = self.svm.latest_blockhash();
        let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);

        let mut signers: Vec<&Keypair> = vec![payer];
        signers.extend_from_slice(extra_signers);

        let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &signers).unwrap();
        self.svm.send_transaction(tx)
    }

    pub fn initialize(&mut self) -> TransactionResult {
        let ix = Instruction::new_with_bytes(
            oracle::id(),
            &oracle::instruction::Initialize {}.data(),
            oracle::accounts::Initialize {
                oracle: self.oracle_pda,
                admin: self.admin.pubkey(),
                system_program: system_program::ID,
            }
            .to_account_metas(None),
        );
        let admin = self.admin.insecure_clone();
        self.send(ix, &admin, &[])
    }

    pub fn update_price(&mut self, signer: &Keypair, new_price: u64) -> TransactionResult {
        let ix = Instruction::new_with_bytes(
            oracle::id(),
            &oracle::instruction::UpdatePrice { new_price }.data(),
            oracle::accounts::UpdatePrice {
                oracle: self.oracle_pda,
                admin: signer.pubkey(),
            }
            .to_account_metas(None),
        );
        self.send(ix, signer, &[])
    }

    pub fn read_oracle(&self) -> OracleState {
        let account = self
            .svm
            .get_account(&self.oracle_pda)
            .expect("oracle account must exist");
        let mut data: &[u8] = &account.data;
        OracleState::try_deserialize(&mut data).unwrap()
    }

    pub fn current_slot(&self) -> u64 {
        self.svm
            .get_sysvar::<anchor_lang::solana_program::clock::Clock>()
            .slot
    }

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
