use log::{error, info};
use crate::common::node_configs::{ChainConfiguration, ContractConfiguration, StoreConfiguration};
use crate::common::node_error::NodeError;
use crate::models::bridge_transaction_model::BridgeTxRecord;
use crate::services::execute_service::ExecuteService;
use crate::utils::store_util::{create_one, create_pool, PgConnectionPool};
use crate::utils::time_util;

pub struct Filter {
    execute_service: Option<ExecuteService>,
    store_config: Option<StoreConfiguration>,
    contract_config: Option<ContractConfiguration>,
}

impl Filter {
    pub fn new() -> Self {
        Self {
            execute_service: None,
            store_config: None,
            contract_config: None,
        }
    }

    pub fn store(mut self, store_config: &StoreConfiguration) -> Self {
        self.store_config = Some(store_config.clone());

        self
    }

    pub fn contract(mut self, contract_config: &ContractConfiguration) -> Self {
        self.contract_config = Some(contract_config.clone());

        self
    }

    pub fn start(&mut self) {
        if let Err(e) = self.connect_execute() {
            error!("{:?}", e);
        };

        self.start_scan_bridge_tx();
    }


    fn connect_execute(&mut self) -> Result<(), NodeError> {
        let execute_service = ExecuteService::new(&self.store_config.clone().unwrap(), &self.contract_config.clone().unwrap(), true)?;

        self.execute_service = Some(execute_service);

        Ok(())
    }

    pub fn start_scan_bridge_tx(&mut self) {
        
        let execute_service = self.execute_service.as_mut().unwrap();
        loop {
            // 获取最后处理的区块高度
            let last_slot = execute_service.get_last_slot().unwrap();
            //let max_slot = execute_service.get_max_slot().unwrap(); //todo tmp del
            let max_slot: i64 = 10;
            let initial_slot = execute_service.get_initial_slot().unwrap();
            if max_slot - 1 <= last_slot {
                info!("all slots are submitted. last slot: {:?} max slot: {:?}", last_slot.clone(),max_slot.clone());
                time_util::sleep_seconds(1);
                continue;
            }

            let start_slot = std::cmp::max(last_slot + 1, initial_slot);
            let end_slot = max_slot - 1;

            let bridge_txs: Vec<BridgeTxRecord> = execute_service.filter_bridge_tx(start_slot.clone(), end_slot.clone()).unwrap();
            
            if !bridge_txs.is_empty() {
                let count = execute_service.insert_bridge_txs(bridge_txs).unwrap();
                info!("insert {:?} bridge txs into pgdb", count);
            }
            execute_service.update_last_slot(max_slot);
        }
    }
}

