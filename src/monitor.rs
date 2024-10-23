use itertools::chain;
use log::error;

use crate::{common::{node_configs::{ChainConfiguration, StoreConfiguration}, node_error::NodeError}, services::{chain_service::ChainService, execute_service::ExecuteService}};

pub struct Monitor {
    execute_service: Option<ExecuteService>,
    chain_service: Option<ChainService>,
    store_config: Option<StoreConfiguration>,
    chain_config: Option<ChainConfiguration>,
}

impl Monitor {
    pub fn new() -> Self {
        Self { 
            execute_service:None,
            chain_service: None, 
            store_config: None,
            chain_config: None,
         }
    }

    pub fn load_store_config(mut self, store_config: &StoreConfiguration) -> Self {
        self.store_config = Some(store_config.clone());
        self
    }

    pub fn load_chain_config(mut self, chain_config: &ChainConfiguration) -> Self {
        self.chain_config = Some(chain_config.clone());
        self
    }

    pub fn connect_execute(&mut self) -> Result<(), NodeError> {
        let execute_service = ExecuteService::new(&self.store_config.clone().unwrap())?;
        
        self.execute_service = Some(execute_service);

        Ok(())
    }

    pub fn connect_chain(&mut self) -> Result<(), NodeError> {
        let chain_service = ChainService::new(&self.chain_config.clone().unwrap()).unwrap();
        
        self.chain_service = Some(chain_service);

        Ok(())
    }

    pub fn read_latest_slot_from_l1(&mut self) {
        
    }

    pub fn start(&mut self) -> Result<(), NodeError> {
        if let Err(e) = self.connect_execute(){
            error!("{:?}", e);
        }

        if let Err(e) = self.connect_chain() {
            error!("{:?}", e);
        }

        loop {
            // check rootmgr latest slot
            self.read_latest_slot_from_l1();
            
            // todo
        }
    }
}