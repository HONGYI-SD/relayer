use log::{error, info};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;

use crate::contract::wrap_slot::WrapSlot;

pub const BRIEF_PDA_SEED: &[u8] = b"fraud_proof_brief";
pub const TALLY_PDA_SEED: &[u8] = b"fraud_proof_tally";

pub const STATE_PDA_SEED: &[u8] = b"fraud_proof_state";


pub struct ChainBasicService<'a> {
    pub rpc_client: &'a RpcClient,
}

impl ChainBasicService<'_> {
    pub fn request_airdrop(&self, pubkey: &Pubkey, lamports: u64) -> bool {
        match self.rpc_client.request_airdrop(pubkey, lamports) {
            Ok(sig) => {
                loop {
                    if let Ok(confirmed) = self.rpc_client.confirm_transaction(&sig) {
                        if confirmed {
                            info!("requesting airdrop. Transaction: {:?}    Status: {:?}", sig, confirmed);
                            break;
                        }
                    }
                }
                true
            }
            Err(err) => {
                error!("requesting airdrop. Error: {:?}", err);
                false
            }
        }
    }

    pub fn find_state_account_address(program_id: &Pubkey) -> (Pubkey, u8) {
        return Pubkey::find_program_address(&[STATE_PDA_SEED], program_id);
    }

    pub fn find_tally_account_address(program_id: &Pubkey) -> (Pubkey, u8) {
        return Pubkey::find_program_address(&[TALLY_PDA_SEED], program_id);
    }

    pub fn find_brief_account_address(program_id: &Pubkey, wrap_slot: WrapSlot) -> (Pubkey, u8) {
        let slot_bytes = wrap_slot.slot.clone().to_le_bytes();
        return Pubkey::find_program_address(&[BRIEF_PDA_SEED, slot_bytes.as_ref()], program_id);
    }
}
