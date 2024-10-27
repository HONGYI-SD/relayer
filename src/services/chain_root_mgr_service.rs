use std::str::FromStr;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;

use crate::contract::wrap_slot::{self, WrapSlot};

use super::chain_basic_service::ChainBasicService;

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct SlotsAccount {
    pub authority: Pubkey,
    pub initialized: bool,
    pub slots: Vec<u64>,
}

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct RootsInfo {
    pub slot: u64,
    pub merkle_tree_root: [u8; 32],
    pub world_state_root: [u8; 32],
}
pub struct ChainRootMgrService<'a> {
    pub rpc_client: &'a RpcClient,
    pub program_id: &'a Pubkey,
    pub slots_acc_pubkey: &'a Pubkey,
}

impl ChainRootMgrService<'_> {
    pub fn fetch_all_slots(&self) -> Option<Vec<u64>> {
        let slots_acc_data = self.rpc_client.get_account_data(self.slots_acc_pubkey).unwrap();
        
        let all_slots = SlotsAccount::deserialize(&mut &slots_acc_data[8..]).unwrap();
        Some(all_slots.slots.clone())
    }

    pub fn fetch_roots_by_slot(&self, slot: u64) -> Option<RootsInfo> {
        let wrap_slot: WrapSlot = WrapSlot { slot };
        let roots_pda = self.find_roots_account_address(wrap_slot);
        let roots_acc_data = self.rpc_client.get_account_data(&roots_pda).unwrap();
        let roots_info = RootsInfo::deserialize(&mut &roots_acc_data[8..]).unwrap();
        //assert_eq!(slot, roots_info.slot, "slot error, slot: {}, roots_info.slot: {}", slot, roots_info.slot); //todo tmp del
        Some(roots_info)
    }

    pub fn find_roots_account_address(&self, wrap_slot: WrapSlot) -> Pubkey {
        return ChainBasicService::find_roots_account_address(self.program_id, wrap_slot.to_owned()).0;
    }
}