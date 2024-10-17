use borsh::BorshDeserialize;
use log::{error, info};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    message::Message, pubkey::Pubkey, signer::Signer, transaction::Transaction,
};
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::signature::Keypair;

use crate::contract::chain_tally::ChainTally;
use crate::contract::wrap_slot::WrapSlot;
use crate::services::chain_basic_service::ChainBasicService;

const CREATE_TALLY_ACCOUNT_DISCRIMINANT: [u8; 8] = [240, 223, 189, 215, 160, 223, 69, 209];

const DESTROY_TALLY_ACCOUNT_DISCRIMINANT: [u8; 8] = [223, 30, 135, 222, 162, 92, 71, 135];

pub struct ChainTallyService<'a> {
    pub rpc_client: &'a RpcClient,
    pub program_id: &'a Pubkey,
    pub payer: &'a Keypair,
}

impl ChainTallyService<'_> {
    pub fn create_tally_account(&self) -> bool {
        let mut is_success: bool = true;
        let payer = self.payer;
        let tally_account_address: Pubkey = self.find_tally_account_address();
        let state_account_address: Pubkey = self.find_state_account_address();
        let last_blockhash = self.rpc_client.get_latest_blockhash().unwrap();

        let tally_account: AccountMeta = AccountMeta::new(tally_account_address, false);
        let state_account: AccountMeta = AccountMeta::new(state_account_address, false);
        let payer_account: AccountMeta = AccountMeta::new_readonly(payer.pubkey(), true);
        let system_program_account: AccountMeta = AccountMeta::new_readonly(solana_sdk::system_program::ID, false);

        let account_metas: Vec<AccountMeta> = vec![
            tally_account.to_owned(),
            state_account.to_owned(),
            payer_account.to_owned(),
            system_program_account.to_owned(),
        ];

        let ix = Instruction::new_with_borsh(
            *self.program_id,
            &(CREATE_TALLY_ACCOUNT_DISCRIMINANT),
            account_metas.to_owned(),
        );
        let message = Message::new(&[ix], Some(&payer.pubkey()));
        let mut tx = Transaction::new_unsigned(message);
        tx.sign(&[&payer], last_blockhash);

        let ret = self.rpc_client.send_and_confirm_transaction_with_spinner(&tx);

        if ret.is_ok() {
            is_success = true;
            info!("create tally account success. ret: {:?}", ret);
        } else {
            is_success = false;
            error!("create tally account fail. ret: {:?}", ret);
        }

        return is_success;
    }

    pub fn destroy_tally_account(&self) -> bool {
        let mut is_success: bool = true;
        let payer = self.payer;
        let tally_account_address: Pubkey = self.find_tally_account_address();
        let state_account_address: Pubkey = self.find_state_account_address();
        let last_blockhash = self.rpc_client.get_latest_blockhash().unwrap();

        let tally_account: AccountMeta = AccountMeta::new(tally_account_address, false);
        let state_account: AccountMeta = AccountMeta::new(state_account_address, false);
        let payer_account: AccountMeta = AccountMeta::new_readonly(payer.pubkey(), true);
        let system_program_account: AccountMeta = AccountMeta::new_readonly(solana_sdk::system_program::ID, false);


        let account_metas: Vec<AccountMeta> = vec![
            tally_account.to_owned(),
            state_account.to_owned(),
            payer_account.to_owned(),
            system_program_account.to_owned(),
        ];

        let ix = Instruction::new_with_borsh(
            *self.program_id,
            &(DESTROY_TALLY_ACCOUNT_DISCRIMINANT),
            account_metas.to_owned(),
        );
        let message = Message::new(&[ix], Some(&payer.pubkey()));
        let mut tx = Transaction::new_unsigned(message);
        tx.sign(&[&payer], last_blockhash);

        let ret = self.rpc_client.send_and_confirm_transaction_with_spinner(&tx);

        if ret.is_ok() {
            is_success = true;
            info!("destroy tally account success. ret: {:?}", ret);
        } else {
            is_success = false;
            error!("destroy tally account fail. ret: {:?}", ret);
        }

        return is_success;
    }


    pub fn is_tally_account_exist(&self) -> bool {
        let tally_account_address = self.find_tally_account_address();

        let is_ok = self.rpc_client
            .get_account(&tally_account_address)
            .is_ok();

        return is_ok;
    }


    //current max wrap slot
    pub fn get_max_wrap_slot(&self) -> Option<WrapSlot> {
        let tally_account_address = self.find_tally_account_address();
        let mut tally_account_data = self.rpc_client
            .get_account_data(&tally_account_address)
            .expect("get account data fail");
        info!(" tally_account_data: {:?}", tally_account_data);
        // solana 账户数据，前8个字节通常是账户的数据长度（lamports 和租约信息）
        let tally = ChainTally::deserialize(&mut &tally_account_data[8..]).unwrap();
        info!("tally: {:?}", tally);

        let max_wrap_slot = WrapSlot {
            slot: tally.present_slot
        };

        return Some(max_wrap_slot);
    }

    pub fn find_state_account_address(&self) -> Pubkey {
        return ChainBasicService::find_state_account_address(self.program_id).0;
    }

    pub fn find_tally_account_address(&self) -> Pubkey {
        return ChainBasicService::find_tally_account_address(self.program_id).0;
    }
}
