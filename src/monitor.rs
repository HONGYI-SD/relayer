use log::{error, info};
use dd_merkle_tree::{MerkleTree, HashingAlgorithm};
use crate::{common::{node_configs::{ChainConfiguration, ContractConfiguration, StoreConfiguration}, node_error::NodeError}, services::{chain_service::ChainService, execute_service::ExecuteService}, utils::{time_util, uuid_util::generate_uuid}};

pub struct Monitor {
    execute_service: Option<ExecuteService>,
    chain_service: Option<ChainService>,
    store_config: Option<StoreConfiguration>,
    chain_config: Option<ChainConfiguration>,
    contract_config: Option<ContractConfiguration>,
    local_tree: Option<MerkleTree>,
}

impl Monitor {
    pub fn new() -> Self {
        Self { 
            execute_service:None,
            chain_service: None, 
            store_config: None,
            chain_config: None,
            contract_config: None,
            local_tree: None,
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

    pub fn load_contract_config(mut self, conctract_config: &ContractConfiguration) -> Self {
        self.contract_config = Some(conctract_config.clone());

        self
    }

    pub fn connect_execute(&mut self) -> Result<(), NodeError> {
        let execute_service = ExecuteService::new(&self.store_config.clone().unwrap(), &self.contract_config.clone().unwrap())?;
        
        self.execute_service = Some(execute_service);

        Ok(())
    }

    pub fn connect_chain(&mut self) -> Result<(), NodeError> {
        let chain_service = ChainService::new(&self.chain_config.clone().unwrap()).unwrap();
        
        self.chain_service = Some(chain_service);

        Ok(())
    }

    pub fn start(&mut self) -> Result<(), NodeError> {
        if let Err(e) = self.connect_execute(){
            error!("{:?}", e);
        }

        if let Err(e) = self.connect_chain() {
            error!("{:?}", e);
        }

        self.local_tree = Some(MerkleTree::new(HashingAlgorithm::Sha256d, 32));
        let chain_service = self.chain_service.as_mut().unwrap();
        let execute_service = self.execute_service.as_mut().unwrap();
        let local_tree = self.local_tree.as_mut().unwrap();

        let local_last_slot = execute_service.get_last_slot_for_monitor().unwrap();
        if local_last_slot > 0 {
            let old_hashes = execute_service.brige_txs_hashes(0, local_last_slot).unwrap();
            let _ = local_tree.add_hashes(old_hashes.into_iter().map(|str_hash| hex::decode(str_hash).expect("failed to decode hex string")).collect());
        }
        
        loop {
            // check rootmgr latest slot
            let chain_last_slot = chain_service.get_latest_slot().unwrap_or_default();
            if !(chain_last_slot > local_last_slot as u64) {
                info!("there is no slot update on chain. local last slot: {:?}, chain last slot: {:?}", local_last_slot.clone(),chain_last_slot.clone());
                time_util::sleep_seconds(1);
                continue;
            }
            
            let mut bridge_txs = execute_service.bridge_tx_range(local_last_slot, chain_last_slot as i64).unwrap();
            let bridge_txs_hashes: Vec<_>= bridge_txs.clone().into_iter().map(|bt| {bt.tx_hash}).collect();
            let _ = local_tree.add_hashes(bridge_txs_hashes.into_iter().map(|str_hash| hex::decode(str_hash).expect("failed to decode hex string")).collect());
            
            let _ = local_tree.merklize();

            let local_mt_root = local_tree.get_merkle_root().unwrap();
            let chain_roots_info = chain_service.get_roots_info_by_slot(chain_last_slot).unwrap();
            if chain_roots_info.merkle_tree_root.to_vec() != local_mt_root {
                error!("local merkle tree is different to the tree on chain");
                break Err(NodeError::new(generate_uuid(), "local merkle tree is different to the tree on chain".to_string()));
            }

            let _ = bridge_txs.iter_mut().map(| bt| {
                let proof = local_tree.merkle_proof_hash(hex::decode(bt.clone().tx_hash).unwrap()).unwrap();
                bt.proof = hex::encode(proof.get_pairing_hashes());
            });

            for bt in bridge_txs {
                execute_service.bridge_tx_update(bt).unwrap();
            }

            execute_service.update_last_slot_for_monitor(chain_last_slot as i64);
        }
    }
}