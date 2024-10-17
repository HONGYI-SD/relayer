use log::{error, info};
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};
use std::str::FromStr;

use crate::common::node_configs::ChainConfiguration;
use crate::common::node_error::NodeError;
use crate::contract::chain_brief::ChainBrief;
use crate::contract::wrap_slot::WrapSlot;
use crate::services::chain_basic_service::ChainBasicService;
use crate::services::chain_brief_service::ChainBriefService;
use crate::services::chain_state_service::ChainStateService;
use crate::services::chain_tally_service::ChainTallyService;

pub struct ChainService {
    rpc_client: RpcClient,
    chain_config: ChainConfiguration,
}

impl ChainService {
    pub fn new(config: &ChainConfiguration) -> Result<Self, NodeError> {
        let rpc_client: RpcClient = RpcClient::new_with_commitment(config.clone().url, CommitmentConfig::confirmed());

        info!("Created RpcClient.");

        Ok(Self {
            rpc_client: rpc_client,
            chain_config: config.clone(),
        })
    }

    pub fn get_role(&self, keypair: &str) -> Option<Keypair> {
        let mut is_success: bool = true;

        let role = Keypair::from_base58_string(keypair);
        let chain_basic_service = ChainBasicService {
            rpc_client: &self.rpc_client,
        };

        is_success = chain_basic_service.request_airdrop(&role.pubkey().clone(), 10_000_000_000);
        if is_success {
            info!("request airdrop success. role pubkey: {:?}; role keypair: {:?}",
                role.pubkey().to_string(), role.to_base58_string());
            return Some(role);
        } else {
            error!("request airdrop fail. role pubkey: {:?}; role keypair: {:?}",
                role.pubkey().to_string(), role.to_base58_string());
            return None;
        }
    }

    pub fn create_state_account(&mut self) -> bool {
        let mut is_success: bool = true;

        let fraud_proof_native_program_id_binding = Pubkey::from_str(&self.chain_config.clone().fraud_proof_native_program_id);
        let fraud_proof_native_program_id = fraud_proof_native_program_id_binding.as_ref().unwrap();

        let execute_node = self.get_role(&self.chain_config.clone().execute_keypair).unwrap();

        let chain_state_service = ChainStateService {
            rpc_client: &self.rpc_client,
            program_id: fraud_proof_native_program_id,
            payer: &execute_node,
        };

        let is_state_exist = chain_state_service.is_state_account_exist();
        if !is_state_exist {
            info!("state account is not exist.");
            is_success = chain_state_service.initialize();
            if !is_success {
                error!("initialize fail.");
                return is_success.clone();
            } else {
                info!("initialize success.");
            }
        } else {
            info!("state account is already exist.");
        }

        return is_success.clone();
    }

    pub fn create_tally_account(&mut self) -> bool {
        let mut is_success: bool = true;

        let fraud_proof_native_program_id_binding = Pubkey::from_str(&self.chain_config.clone().fraud_proof_native_program_id);
        let fraud_proof_native_program_id = fraud_proof_native_program_id_binding.as_ref().unwrap();

        let execute_node = self.get_role(&self.chain_config.clone().execute_keypair).unwrap();

        let chain_tally_service = ChainTallyService {
            rpc_client: &self.rpc_client,
            program_id: fraud_proof_native_program_id,
            payer: &execute_node,
        };

        let is_tally_exist = chain_tally_service.is_tally_account_exist();
        if !is_tally_exist {
            info!("tally account is not exist.");
            is_success = chain_tally_service.create_tally_account();
            if !is_success {
                error!("create tally account fail.");
                return is_success.clone();
            } else {
                info!("create tally account success.");
            }
        } else {
            info!("tally account is already exist.");
        }

        return is_success.clone();
    }


    pub fn create_brief_account(&mut self, brief: ChainBrief) -> bool {
        let mut is_success: bool = true;

        let fraud_proof_native_program_id_binding = Pubkey::from_str(&self.chain_config.clone().fraud_proof_native_program_id);
        let fraud_proof_native_program_id = fraud_proof_native_program_id_binding.as_ref().unwrap();

        let execute_node = self.get_role(&self.chain_config.clone().execute_keypair).unwrap();

        let chain_brief_service = ChainBriefService {
            rpc_client: &self.rpc_client,
            program_id: fraud_proof_native_program_id,
            payer: &execute_node,
        };

        let wrap_slot: WrapSlot = WrapSlot {
            slot: brief.slot,
        };
        
        let is_brief_exist = chain_brief_service.is_brief_account_exist(wrap_slot.clone());
        if is_brief_exist {
            info!("brief account is already exist. slot: {:?}", wrap_slot.clone());
            return is_success.clone();
        }
        info!("brief account is not exist. slot: {:?},brief: {:?}", wrap_slot.clone(), brief);
        is_success = chain_brief_service.create_brief_account(wrap_slot.clone(), brief.clone());
        if is_success {
            let is_brief_exist = chain_brief_service.is_brief_account_exist(wrap_slot.clone());
            if is_brief_exist {
                let save_brief = chain_brief_service.fetch_brief_account(wrap_slot.clone()).unwrap();
                info!("create brief account success. slot: {:?}, brief: {:?}", wrap_slot.clone(), save_brief.clone());
            } else {
                error!("create brief account fail. slot: {:?}", wrap_slot.clone());
                is_success = false;
                return is_success.clone();
            }
        } else {
            error!("create brief account fail. slot: {:?}", wrap_slot.clone());
            return is_success.clone();
        }

        return is_success.clone();
    }
}

