use std::str::FromStr;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct SlotAccount {
    pub slots: Vec<u64>,
}

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct RootsInfo {
    pub merkle_root: [u8; 32],
    pub world_state_root: [u8; 32],
}
pub struct ChainRootMgrService<'a> {
    pub rpc_client: &'a RpcClient,
    pub program_id: &'a Pubkey,
}

impl ChainRootMgrService<'_> {
    pub fn fetch_all_slots(&self) -> Option<Vec<u64>> {
        let slot_account_pubkey = Pubkey::from_str("todo fixed account").unwrap();
        let slot_acc_data = self.rpc_client.get_account_data(&slot_account_pubkey).unwrap();
        let all_slots = SlotAccount::try_from_slice(&slot_acc_data).unwrap();
        Some(all_slots.slots.clone())
    }

    pub fn fetch_roots_by_slot(&self, _slot: u64) -> Option<RootsInfo> {
        let slot_pda_pubkey = Pubkey::from_str("todo pda account").unwrap();
        let roots_acc_data = self.rpc_client.get_account_data(&slot_pda_pubkey).unwrap();
        let roots_info = RootsInfo::try_from_slice(&roots_acc_data).unwrap();
        Some(roots_info)
    }
}