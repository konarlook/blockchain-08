use anchor_client::anchor_lang::{prelude::Pubkey, solana_program::bpf_loader_upgradeable};

pub fn program_data_address(program_id: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[program_id.as_ref()], &bpf_loader_upgradeable::ID).0
}
