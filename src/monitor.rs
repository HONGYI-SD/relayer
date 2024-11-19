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
        let execute_service = ExecuteService::new(&self.store_config.clone().unwrap(), &self.contract_config.clone().unwrap(), false)?;
        
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
        let mut local_tree_leaf_num = 0;

        // get earliest no proof bridge tx
        let mut earliest_no_proof_tx_slot: i64 = 0;
        if let Ok(earliest_no_proof_tx) = execute_service.get_ealiest_no_proof_bridge_tx_from_pg_for_monitor() {
            earliest_no_proof_tx_slot = earliest_no_proof_tx.slot;
        }
        info!("dong: earliest no proof tx slot: {}", earliest_no_proof_tx_slot);

        let mut max_has_proof_tx_slot: i64 = 0;
        if let Ok(last_has_proof_tx) = execute_service.get_last_has_proof_bridge_tx_from_pg_for_monitor() {
            max_has_proof_tx_slot = last_has_proof_tx.slot;
        }
        //let last_has_proof_tx_slot = execute_service.get_last_slot_from_rkdb_for_monitor().unwrap();
        info!("dong: local_last_slot: {}", max_has_proof_tx_slot);
        if max_has_proof_tx_slot > 0 {
            let old_hashes = execute_service.brige_txs_hashes(0, max_has_proof_tx_slot).unwrap();
            if old_hashes.len() != 0 {
                local_tree_leaf_num = old_hashes.len() - 1;
            }
            info!("dong: old_hashes {:?}", old_hashes);
            let _ = local_tree.add_hashes(old_hashes);
            
        }
        
        loop {
            // check rootmgr latest slot
            
            let chain_all_slots = chain_service.get_all_slots_from_chain().unwrap_or_default();
            //let chain_all_slots = vec![52833];
            if chain_all_slots.len() == 0 as usize {
                info!("there is no slots info on chain, waitting...");
                time_util::sleep_seconds(1);
                continue;
            }
            info!("dong: chain_all_slots: {:?}", chain_all_slots);
            let chain_last_slot = chain_all_slots[chain_all_slots.len() - 1];
            info!("dong: local_last_slot: {}, chain_last_slot: {}", max_has_proof_tx_slot, chain_last_slot);

            if !(chain_last_slot > max_has_proof_tx_slot as u64) {
                info!("there is no slot update on chain. local last slot: {:?}, chain last slot: {:?}", max_has_proof_tx_slot.clone(),chain_last_slot.clone());
                time_util::sleep_seconds(1);
                continue;
            }
            
            let chain_sub_slots: Vec<u64> = chain_all_slots.iter().filter(|&&s| s > max_has_proof_tx_slot as u64).cloned().collect();
            let mut tmp_start_slot = max_has_proof_tx_slot;
            for tmp_slot in chain_sub_slots {
                let mut bridge_txs = execute_service.bridge_tx_range(tmp_start_slot, tmp_slot as i64).unwrap();
                let bridge_txs_hashes: Vec<Vec<u8>>= bridge_txs.clone().into_iter().map(|bt| {bt.tx_info_hash}).collect();
                info!("dong: bridge_txs_hashes {:?}", bridge_txs_hashes);
                let _ = local_tree.add_hashes(bridge_txs_hashes).unwrap();
                
                let _ = local_tree.merklize().unwrap();
    
                let local_mt_root = local_tree.get_merkle_root().unwrap();
                let chain_roots_info = chain_service.get_roots_info_by_slot(tmp_slot).unwrap();
                // todo tmp del
                if chain_roots_info.merkle_tree_root.to_vec() != local_mt_root {
                    error!("local merkle tree is different to the tree on chain, chain merkle tree root: {:?}, local root: {:?}", chain_roots_info.merkle_tree_root.to_vec(), local_mt_root);
                    return  Err(NodeError::new(
                        generate_uuid(), 
                        "local merkle tree is different to the tree on chain".to_string()
                    ));
                }
    
                info!("dong monitor: 11 bridge_txs: {:?}", bridge_txs);
                let _ = bridge_txs.iter_mut().for_each(| bt| {
                    //let proof = local_tree.merkle_proof_hash(bt.clone().tx_info_hash).unwrap();
                    let proof = local_tree.merkle_proof_index(local_tree_leaf_num).unwrap();
                    bt.proof = hex::encode(proof.get_pairing_hashes());
                    bt.is_generated_proof = true;
                    bt.current_mt_root = local_mt_root.clone();
                    local_tree_leaf_num += 1;
                });
    
                info!("dong monitor: 22 bridge_txs: {:?}", bridge_txs);
                
                let _ = bridge_txs.iter().for_each(|bt| {
                    info!("updata tx: {:?}", bt.clone());
                    execute_service.bridge_tx_update(bt.clone()).unwrap();
                });
                max_has_proof_tx_slot = tmp_slot as i64;
                tmp_start_slot = tmp_slot as i64;
            }
            //execute_service.update_last_slot_to_rkdb_for_monitor(chain_last_slot as i64);
        }
    }
}