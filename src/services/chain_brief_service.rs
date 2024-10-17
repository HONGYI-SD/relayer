use borsh::BorshDeserialize;
use log::{error, info};
use solana_client::rpc_client::RpcClient;
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::message::Message;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::Transaction;

use crate::contract::chain_brief::ChainBrief;
use crate::contract::wrap_slot::WrapSlot;
use crate::services::chain_basic_service::ChainBasicService;

const CREATE_BRIEF_ACCOUNT_DISCRIMINANT: [u8; 8] = [33, 225, 8, 221, 25, 134, 30, 62];

const DESTROY_BRIEF_ACCOUNT_DISCRIMINANT: [u8; 8] = [222, 100, 168, 78, 26, 165, 114, 80];


pub struct ChainBriefService<'a> {
    pub rpc_client: &'a RpcClient,
    pub program_id: &'a Pubkey,
    pub payer: &'a Keypair,
}

impl ChainBriefService<'_> {
    pub fn create_brief_account(&self, wrap_slot: WrapSlot, chain_brief: ChainBrief) -> bool {
        let mut is_success: bool = true;
        let payer = self.payer;
        let brief_account_address: Pubkey = self.find_brief_account_address(wrap_slot.to_owned());
        let tally_account_address: Pubkey = self.find_tally_account_address();
        let state_account_address: Pubkey = self.find_state_account_address();
        let last_blockhash = self.rpc_client.get_latest_blockhash().unwrap();


        let brief_account: AccountMeta = AccountMeta::new(brief_account_address, false);
        let tally_account: AccountMeta = AccountMeta::new(tally_account_address, false);
        let state_account: AccountMeta = AccountMeta::new(state_account_address, false);
        let payer_account: AccountMeta = AccountMeta::new_readonly(payer.pubkey(), true);
        let system_program_account: AccountMeta = AccountMeta::new_readonly(solana_sdk::system_program::ID, false);


        let account_metas: Vec<AccountMeta> = vec![
            brief_account.to_owned(),
            tally_account.to_owned(),
            state_account.to_owned(),
            payer_account.to_owned(),
            system_program_account.to_owned(),
        ];

        let ix = Instruction::new_with_borsh(
            *self.program_id,
            &(CREATE_BRIEF_ACCOUNT_DISCRIMINANT, chain_brief.clone()),
            account_metas.to_owned(),
        );
        let message = Message::new(&[ix], Some(&payer.pubkey()));
        let mut tx = Transaction::new_unsigned(message);
        tx.sign(&[&payer], last_blockhash);

        let ret = self.rpc_client.send_and_confirm_transaction_with_spinner(&tx);

        if ret.is_ok() {
            is_success = true;
            info!("create brief account success. ret: {:?}", ret);
        } else {
            is_success = false;
            error!("create brief account fail. ret: {:?}", ret);
        }


        return is_success;
    }

    pub fn destroy_brief_account(&self, wrap_slot: WrapSlot) -> bool {
        let mut is_success: bool = true;
        let payer = self.payer;
        let brief_account_address: Pubkey = self.find_brief_account_address(wrap_slot.to_owned());
        let tally_account_address: Pubkey = self.find_tally_account_address();
        let state_account_address: Pubkey = self.find_state_account_address();
        let last_blockhash = self.rpc_client.get_latest_blockhash().unwrap();

        let brief_account: AccountMeta = AccountMeta::new(brief_account_address, false);
        let tally_account: AccountMeta = AccountMeta::new(tally_account_address, false);
        let state_account: AccountMeta = AccountMeta::new(state_account_address, false);
        let payer_account: AccountMeta = AccountMeta::new_readonly(payer.pubkey(), true);
        let system_program_account: AccountMeta = AccountMeta::new_readonly(solana_sdk::system_program::ID, false);

        let account_metas: Vec<AccountMeta> = vec![
            brief_account.to_owned(),
            tally_account.to_owned(),
            state_account.to_owned(),
            payer_account.to_owned(),
            system_program_account.to_owned(),
        ];

        let ix = Instruction::new_with_borsh(
            *self.program_id,
            &(DESTROY_BRIEF_ACCOUNT_DISCRIMINANT),
            account_metas.to_owned(),
        );
        let message = Message::new(&[ix], Some(&payer.pubkey()));
        let mut tx = Transaction::new_unsigned(message);
        tx.sign(&[&payer], last_blockhash);

        let ret = self.rpc_client.send_and_confirm_transaction_with_spinner(&tx);

        if ret.is_ok() {
            is_success = true;
            info!("destroy brief account success. ret: {:?}", ret);
        } else {
            is_success = false;
            error!("destroy brief account fail. ret: {:?}", ret);
        }

        return is_success;
    }

    pub fn fetch_brief_account(&self, wrap_slot: WrapSlot) -> Option<ChainBrief> {
        let brief_account_address =
            self.find_brief_account_address(wrap_slot.to_owned());

        let mut brief_account_data = self.rpc_client
            .get_account_data(&brief_account_address)
            .expect("get account data fail");
        info!(" brief_account_data: {:?}", brief_account_data);
        // solana 账户数据，前8个字节通常是账户的数据长度（lamports 和租约信息）
        let brief = ChainBrief::deserialize(&mut &brief_account_data[8..]).unwrap();
        info!("brief: {:?}", brief);

        return Some(brief);
    }

    pub fn is_brief_account_exist(&self, wrap_slot: WrapSlot) -> bool {
        let brief_account_address = self.find_brief_account_address(wrap_slot.clone());

        let is_ok = self.rpc_client
            .get_account(&brief_account_address)
            .is_ok();

        return is_ok;
    }

    pub fn find_state_account_address(&self) -> Pubkey {
        return ChainBasicService::find_state_account_address(self.program_id).0;
    }

    pub fn find_tally_account_address(&self) -> Pubkey {
        return ChainBasicService::find_tally_account_address(self.program_id).0;
    }
    pub fn find_brief_account_address(&self, wrap_slot: WrapSlot) -> Pubkey {
        return ChainBasicService::find_brief_account_address(self.program_id, wrap_slot.to_owned()).0;
    }
}
