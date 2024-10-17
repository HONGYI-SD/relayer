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

const INITIALIZE_DISCRIMINANT: [u8; 8] = [175, 175, 109, 31, 13, 152, 155, 237];

pub struct ChainStateService<'a> {
    pub rpc_client: &'a RpcClient,
    pub program_id: &'a Pubkey,
    pub payer: &'a Keypair,
}

impl ChainStateService<'_> {
    pub fn initialize(&self) -> bool {
        let mut is_success: bool = true;
        let payer = self.payer;
        let state_account_address: Pubkey = self.find_state_account_address();
        let last_blockhash = self.rpc_client.get_latest_blockhash().unwrap();

        let state_account: AccountMeta = AccountMeta::new(state_account_address, false);
        let payer_account: AccountMeta = AccountMeta::new_readonly(payer.pubkey(), true);
        let system_program_account: AccountMeta = AccountMeta::new_readonly(solana_sdk::system_program::ID, false);

        let account_metas: Vec<AccountMeta> = vec![
            state_account.to_owned(),
            payer_account.to_owned(),
            system_program_account.to_owned(),
        ];

        let ix = Instruction::new_with_borsh(
            *self.program_id,
            &(INITIALIZE_DISCRIMINANT),
            account_metas.to_owned(),
        );
        let message = Message::new(&[ix], Some(&payer.pubkey()));
        let mut tx = Transaction::new_unsigned(message);
        tx.sign(&[&payer], last_blockhash);

        let ret = self.rpc_client.send_and_confirm_transaction_with_spinner(&tx);

        if ret.is_ok() {
            is_success = true;
            info!("initialize success. ret: {:?}", ret);
        } else {
            is_success = false;
            error!("initialize fail. ret: {:?}", ret);
        }

        return is_success;
    }

    pub fn is_state_account_exist(&self) -> bool {
        let state_account_address = self.find_state_account_address();

        let is_ok = self.rpc_client
            .get_account(&state_account_address)
            .is_ok();

        return is_ok;
    }


    pub fn find_state_account_address(&self) -> Pubkey {
        return ChainBasicService::find_state_account_address(self.program_id).0;
    }
}
