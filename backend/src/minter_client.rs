use anchor_client::{
    anchor_lang::{prelude::Pubkey, solana_program::system_program},
    ClientError, Program, Signer,
};
use anchor_spl::{associated_token, token};
use minter::MinterConfig;
use solana_keypair::Keypair;
use solana_signature::Signature;

use crate::connection::Payer;

pub fn config_address(program_id: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[minter::CONFIG_SEED], program_id).0
}

pub fn mint_authority_address(program_id: &Pubkey, mint: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[minter::MINT_AUTHORITY_SEED, mint.as_ref()], program_id).0
}

pub fn read_config(
    program: &Program<Payer>,
    config: Pubkey,
) -> Result<Option<MinterConfig>, ClientError> {
    match program.account::<MinterConfig>(config) {
        Ok(account) => Ok(Some(account)),
        Err(ClientError::AccountNotFound) => Ok(None),
        Err(err) => Err(err),
    }
}

pub fn initialize(
    program: &Program<Payer>,
    config: Pubkey,
    treasury: Pubkey,
    fee_usd: u64,
) -> Result<Signature, ClientError> {
    program
        .request()
        .accounts(minter::accounts::Initialize {
            config,
            admin: program.payer(),
            treasury,
            system_program: system_program::ID,
        })
        .args(minter::instruction::Initialize { fee_usd })
        .send()
}

pub struct CreateTokenAccounts {
    pub mint: Pubkey,
    pub mint_authority: Pubkey,
    pub creator_ata: Pubkey,
}

pub fn create_token_accounts(program: &Program<Payer>, mint: &Keypair) -> CreateTokenAccounts {
    let mint = mint.pubkey();
    CreateTokenAccounts {
        mint,
        mint_authority: mint_authority_address(&program.id(), &mint),
        creator_ata: associated_token::get_associated_token_address(&program.payer(), &mint),
    }
}

pub fn create_token(
    program: &Program<Payer>,
    config: Pubkey,
    oracle_state: Pubkey,
    treasury: Pubkey,
    mint: &Keypair,
    decimals: u8,
    initial_supply: u64,
) -> Result<Signature, ClientError> {
    let accounts = create_token_accounts(program, mint);
    program
        .request()
        .accounts(minter::accounts::CreateToken {
            payer: program.payer(),
            config,
            oracle: oracle_state,
            treasury,
            mint: accounts.mint,
            mint_authority: accounts.mint_authority,
            payer_ata: accounts.creator_ata,
            token_program: token::ID,
            associated_token_program: associated_token::ID,
            system_program: system_program::ID,
        })
        .args(minter::instruction::CreateToken {
            decimals,
            initial_supply,
        })
        .signer(mint)
        .send()
}
