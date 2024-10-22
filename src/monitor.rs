use itertools::chain;

use crate::{common::{node_configs::ChainConfiguration, node_error::NodeError}, services::chain_service::ChainService};

pub struct Monitor {
    chain_service: Option<ChainService>,
    chain_config: Option<ChainConfiguration>,
}

impl Monitor {
    pub fn new() -> Self {
        Self { 
            chain_service: None, 
            chain_config: None,
         }
    }

    pub fn load_chain_config(mut self, chain_config: &ChainConfiguration) -> Self {
        self.chain_config = Some(chain_config.clone());
        self
    }

    pub fn connect_chain(mut self) -> Self {
        let chain_service = ChainService::new(&self.chain_config.clone().unwrap()).unwrap();
        
        self.chain_service = Some(chain_service);

        self
    }

    pub fn read_latest_slot_from_l1(&mut self) {

    }

    pub fn run(&mut self) -> Result<(), NodeError> {
        loop {
            // check rootmgr latest slot
            self.read_latest_slot_from_l1();
            
            // todo
        }
    }
}