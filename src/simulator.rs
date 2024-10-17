use std::str::FromStr;
use std::time::Duration;

use log::{error, info};
use postgres::Client;
use rand::Rng;
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::hash::Hash;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};

use crate::common::node_configs::{ChainConfiguration, StoreConfiguration};
use crate::common::node_error::NodeError;
use crate::contract::chain_brief::ChainBrief;
use crate::services::chain_basic_service::ChainBasicService;
use crate::services::chain_brief_service::ChainBriefService;
use crate::services::chain_service::ChainService;
use crate::services::chain_state_service::ChainStateService;
use crate::services::chain_tally_service::ChainTallyService;
use crate::services::execute_service::ExecuteService;
use crate::utils::store_util::{create_one, create_pool, PgConnectionPool};
use crate::utils::time_util;
use crate::utils::uuid_util::generate_uuid;

pub struct Simulator {
    execute_service: Option<ExecuteService>,
    chain_service: Option<ChainService>,
    store_config: Option<StoreConfiguration>,
    chain_config: Option<ChainConfiguration>,
}


impl Simulator {
    pub fn new() -> Self {
        Self {
            execute_service: None,
            chain_service: None,
            store_config: None,
            chain_config: None,
        }
    }

    pub fn store(mut self, store_config: &StoreConfiguration) -> Self {
        self.store_config = Some(store_config.clone());
        self
    }

    pub fn chain(mut self, chain_config: &ChainConfiguration) -> Self {
        self.chain_config = Some(chain_config.clone());
        self
    }

    pub fn start(&mut self) {
        if let Err(e) = self.connect_execute() {
            error!("{:?}", e);
        };

        if let Err(e) = self.connect_chain() {
            error!("{:?}", e);
        };


        let is_success = self.start_submit_brief();
        if is_success {
            info!("submit brief success.");
        } else {
            error!("submit brief fail.");
        }
    }


    fn connect_execute(&mut self) -> Result<(), NodeError> {
        let mut execute_service = ExecuteService::new(&self.store_config.clone().unwrap())?;

        match execute_service.check_smt() {
            Ok(flag) => {
                if !flag {
                    error!("check SMT is false");
                    return Err(
                        NodeError::new(generate_uuid(),
                                       "check SMT is false.".to_string(),
                        )
                    );
                }
            }
            Err(e) => {
                error!("check SMT failed. err: {}", e);
                return Err(
                    NodeError::new(generate_uuid(),
                                   "check SMT failed.".to_string(),
                    )
                );
            }
        }

        self.execute_service = Some(execute_service);

        Ok(())
    }


    fn connect_chain(&mut self) -> Result<(), NodeError> {
        let chain_service = ChainService::new(&self.chain_config.clone().unwrap()).unwrap();

        self.chain_service = Some(chain_service);

        Ok(())
    }

    pub fn start_submit_brief(&mut self) -> bool {
        let mut is_success: bool = true;

        let execute_service = self.execute_service.as_mut().unwrap();
        let chain_service = self.chain_service.as_mut().unwrap();

        is_success = chain_service.create_state_account();
        if !is_success {
            error!("create state account fail.");
            return is_success.clone();
        }

        is_success = chain_service.create_tally_account();
        if !is_success {
            error!("create tally account fail.");
            return is_success.clone();
        }

        loop {
            // 获取最后处理的区块高度
            let last_slot = execute_service.get_last_slot().unwrap();
            let max_slot = execute_service.get_max_slot().unwrap();
            let initial_slot = execute_service.get_initial_slot().unwrap();
            if max_slot - 1 <= last_slot {
                info!("all slots are submitted. last slot: {:?} max slot: {:?}", last_slot.clone(),max_slot.clone());
                time_util::sleep_seconds(1);
                continue;
            }
            info!("some slots are not submitted. submit them now. last slot: {:?} max slot: {:?}", last_slot.clone(),max_slot.clone());

            let start_slot = std::cmp::max(last_slot + 1, initial_slot);
            let end_slot = max_slot - 1;
            let briefs: Vec<ChainBrief> = execute_service.generate_briefs(start_slot.clone(), end_slot.clone()).unwrap();

            // send brief to chain
            // The slot 0 and slot 1 are initial of blockchain, we never challenge, so skip them.
            for brief in briefs {
                info!("brief: {:?}", brief);
                is_success = chain_service.create_brief_account(brief.clone());
                if !is_success {
                    error!("create brief account fail. slot: {:?}", brief.clone());
                    continue;
                }
            }
        }
    }
}

