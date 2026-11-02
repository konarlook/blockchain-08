use anchor_client::{
    anchor_lang::{prelude::Pubkey, solana_program::system_program},
    ClientError, Program,
};
use oracle::OracleState;
use solana_compute_budget_interface::ComputeBudgetInstruction;
use solana_signature::Signature;

use crate::connection::Payer;

pub fn state_address(program_id: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[oracle::ORACLE_SEED], program_id).0
}

pub fn read_state(
    program: &Program<Payer>,
    state: Pubkey,
) -> Result<Option<OracleState>, ClientError> {
    match program.account::<OracleState>(state) {
        Ok(account) => Ok(Some(account)),
        Err(ClientError::AccountNotFound) => Ok(None),
        Err(err) => Err(err),
    }
}

pub fn initialize(program: &Program<Payer>, state: Pubkey) -> Result<Signature, ClientError> {
    program
        .request()
        .accounts(oracle::accounts::Initialize {
            oracle: state,
            admin: program.payer(),
            system_program: system_program::ID,
        })
        .args(oracle::instruction::Initialize {})
        .send()
}

pub fn update_price(
    program: &Program<Payer>,
    state: Pubkey,
    new_price: u64,
    priority_fee_micro_lamports: u64,
) -> Result<Signature, ClientError> {
    let mut request = program.request();
    if priority_fee_micro_lamports > 0 {
        request = request.instruction(ComputeBudgetInstruction::set_compute_unit_price(
            priority_fee_micro_lamports,
        ));
    }
    request
        .accounts(oracle::accounts::UpdatePrice {
            oracle: state,
            admin: program.payer(),
        })
        .args(oracle::instruction::UpdatePrice { new_price })
        .send()
}
