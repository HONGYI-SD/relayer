use crate::common::node_configs::{ContractConfiguration, StoreConfiguration};
use crate::common::node_error::NodeError;
use crate::contract::chain_brief::ChainBrief;
use crate::models::account_audit_row::AccountAuditRow;
use crate::models::brief_model::convert_chain_briefs_to_brief_records;
use crate::models::transaction_model::TransactionRow;
use crate::models::bridge_transaction_model::{BridgeTxInfo, BridgeTxRecord, BridgeTxRow, MessageType};
use crate::repositories::account_audit_repo::AccountAuditRepo;
use crate::repositories::block_repo::BlockRepo;
use crate::repositories::bridge_tx_repo::BridgeTxRepo;
use crate::repositories::brief_repo::BriefRepo;
use crate::repositories::chain_repo::ChainRepo;
use crate::repositories::transaction_repo::TransactionRepo;
use crate::utils::store_util::{create_one, create_pool, PgConnectionPool};
use crate::utils::uuid_util::generate_uuid;
use borsh::BorshDeserialize;
use log::{error, info};
use postgres::Client;
use rocksdb::DB;
use solana_clap_utils::nonce;
use solana_sdk::pubkey::{self, Pubkey};
use solana_sdk::signature::Signature;
use std::default;
use std::path::Path;
use std::str::FromStr;
use std::sync::{Arc, RwLock};

pub struct ExecuteService {
    client_pool: PgConnectionPool,
    client_one: Client,
    l2_msg_program_id: String,
    l2_message_fund_account_pubkey: String,
    system_program_id: String,
    rocksdb: Arc<RwLock<DB>>,
    monitor_rocksdb_slot: Arc<RwLock<DB>>,
    initial_slot: u64,
}

impl ExecuteService {
    pub fn new(config: &StoreConfiguration, contract: &ContractConfiguration, is_filter: bool) -> Result<Self, NodeError> {
        let pool = create_pool(
            config.to_owned(),
            10,
        );

        let one = create_one(config.to_owned());

        let l2_msg_program_id = contract.l2_message_program_id.clone();
        let l2_message_fund_account_pubkey = contract.l2_message_fund_account_pubkey.clone();
        let system_program_id = contract.system_program_id.clone();
        if is_filter {
            let slot_dir = Path::new("./relayer/filter/slot");
            let slot_db = DB::open_default(slot_dir).unwrap();
            let rocksdb = Arc::new(RwLock::new(slot_db));
            let monitor_slot_dir = Path::new("./relayer/filter/monitor-slot-tmp");
            let monitor_slot_db = DB::open_default(monitor_slot_dir).unwrap();
            let monitor_rocksdb_slot = Arc::new(RwLock::new(monitor_slot_db));
            info!("Created PostgresClient.");

            Ok(Self {
                client_pool: pool,
                client_one: one,
                l2_msg_program_id,
                l2_message_fund_account_pubkey,
                system_program_id,
                rocksdb,
                monitor_rocksdb_slot,
                initial_slot: 2,
            })
        }else {
            let slot_dir = Path::new("./relayer/monitor/slot-tmp");
            let slot_db = DB::open_default(slot_dir).unwrap();
            let rocksdb = Arc::new(RwLock::new(slot_db));

            let monitor_slot_dir = Path::new("./relayer/monitor/monitor-slot");
            let monitor_slot_db = DB::open_default(monitor_slot_dir).unwrap();
            let monitor_rocksdb_slot = Arc::new(RwLock::new(monitor_slot_db));
            info!("Created PostgresClient.");

            Ok(Self {
                client_pool: pool,
                client_one: one,
                l2_msg_program_id,
                l2_message_fund_account_pubkey,
                system_program_id,
                rocksdb,
                monitor_rocksdb_slot,
                initial_slot: 2,
            })
        }
        
    }

    pub fn get_account_audits(&self, from_slot: i64, to_slot: i64) -> Result<Vec<AccountAuditRow>, NodeError> {
        let repo = AccountAuditRepo { pool: Box::from(self.client_pool.to_owned()) };

        let rows = repo.range(from_slot, to_slot)?;

        Ok(rows)
    }

    pub fn get_transactions(&mut self, from_slot: i64, to_slot: i64) -> Result<Vec<TransactionRow>, NodeError> {
        let mut repo = TransactionRepo { one: &mut self.client_one };

        let rows = repo.range(from_slot, to_slot)?;
        Ok(rows)
    }

    pub fn get_initial_slot(&self) -> Result<i64, NodeError> {
        Ok(self.initial_slot as i64)
    }

    pub fn get_last_slot(&self) -> Result<i64, NodeError> {
        let mut repo = ChainRepo { db: &self.rocksdb };

        let slot = repo.show().unwrap_or(0);

        Ok(slot)
    }

    pub fn get_last_slot_from_rkdb_for_monitor(&self) -> Result<i64, NodeError> {
        let mut repo = ChainRepo{ db: &self.monitor_rocksdb_slot };

        let slot = repo.show().unwrap_or(0);
        
        Ok(slot)
    }

    pub fn get_last_has_proof_bridge_tx_from_pg_for_monitor(&self) -> Result<BridgeTxRecord, NodeError>{
        let repo = BridgeTxRepo{ pool: Box::from(self.client_pool.to_owned()) };
        match repo.get_last_bridge_tx_has_proof() {
            Ok(tx_raw) => {
                Ok(BridgeTxRecord::from(tx_raw))
            }
            Err(err) => {
                Err(err)
            }
        }

    }

    pub fn update_last_slot_to_rkdb_for_monitor(&self, slot: i64) {
        let mut repo = ChainRepo { db: &self.monitor_rocksdb_slot };

        repo.upsert(slot);
    }

    pub fn get_max_slot(&mut self) -> Result<i64, NodeError> {
        let mut repo = BlockRepo { one: &mut self.client_one };

        match repo.show() {
            Ok(row) => {
                Ok(row.slot)
            }
            Err(e) => {
                Ok(0)
            }
        }
    }

    pub fn update_last_slot(&self, slot: i64) {
        let mut repo = ChainRepo { db: &self.rocksdb };

        repo.upsert(slot);
    }

    pub fn insert_briefs(&self, chain_briefs: Vec<ChainBrief>) -> Result<u32, NodeError> {
        let repo = BriefRepo { pool: Box::from(self.client_pool.to_owned()) };

        let brief_records = convert_chain_briefs_to_brief_records(chain_briefs);

        let rows = repo.insert(brief_records)?;

        let count = rows.len() as u32;

        Ok(count)
    }

    pub fn filter_bridge_tx(&mut self, start_slot: i64, end_slot: i64) -> Result<Vec<BridgeTxRecord>, NodeError> {
        if end_slot < start_slot {
            error!("end_slot should greater than or equal start_slot  start_slot: {:?},end_slot: {:?}",
                start_slot,end_slot);
            return Err(
                NodeError::new(generate_uuid(),
                               format!("end_slot should greater than or equal start_slot  start_slot: {:?},end_slot: {:?}",
                                       start_slot, end_slot),
                )
            );
        }

        if start_slot < self.initial_slot as i64 || end_slot < self.initial_slot as i64 {
            error!("start_slot and end_slot should greater than initial_slot  start_slot: {:?}, end_slot: {:?}, initial_slot: {:?}",
                start_slot,end_slot,self.initial_slot);
            return Err(
                NodeError::new(generate_uuid(),
                               format!("start_slot and end_slot should greater than initial_slot  start_slot: {:?}, end_slot: {:?}, initial_slot: {:?}",
                                       start_slot, end_slot, self.initial_slot),
                )
            );
        }

        let transactions = self.get_transactions(start_slot, end_slot)?;
        let mut bridge_txs: Vec<BridgeTxRecord> = Vec::new();

        for transaction in &transactions {
            let msg = transaction.clone().legacy_message.unwrap();
            let pks: Vec<Pubkey> = msg.account_keys.iter().map(|ak| Pubkey::try_from(ak.as_slice()).unwrap()).collect();

            if self.check_bridge_message_pubkeys(&pks) {
                if let Some(bridge_tx_record) = self.txraw_to_bridgetx(transaction, &pks){
                    info!("dong: bridge_tx_record:{:?}", bridge_tx_record);
                    bridge_txs.push(bridge_tx_record);
                }
            }
        }
        
        info!("tx len: {}", bridge_txs.len());
        Ok(bridge_txs)
    }

    pub fn check_bridge_message_pubkeys(&self, pubkeys: &Vec<Pubkey>) -> bool {
        if pubkeys.contains(&Pubkey::from_str(&self.l2_msg_program_id).unwrap()) &&
            pubkeys.contains(&Pubkey::from_str(&self.l2_message_fund_account_pubkey).unwrap()) &&
            pubkeys.contains(&Pubkey::from_str(&self.system_program_id).unwrap()) {
                return true;
            }
        
        return false;
    }

    pub fn txraw_to_bridgetx(&self, tx: &TransactionRow, pubkeys: &Vec<Pubkey>) -> Option<BridgeTxRecord> {
        if tx.meta.inner_instructions.as_ref().unwrap().len() == 0 {
            return None;
        }
        if tx.meta.inner_instructions.as_ref().unwrap()[0].instructions.len() == 0 {
            return None;
        } 
        // system transfer instruction data len is 9 or 12
        let data_len = tx.meta.inner_instructions.as_ref().unwrap()[0].instructions[0].data.len();
        if  data_len < 9 {
            return None;
        }
        if tx.meta.inner_instructions.as_ref().unwrap()[0].instructions[0].accounts.len() != 2 {
            return None;
        }
        let accounts_idx = tx.meta.inner_instructions.as_ref().unwrap()[0].instructions[0].accounts.clone();
        let ix_data = tx.meta.inner_instructions.as_ref().unwrap()[0].instructions[0].data.clone();
        let opcode = ix_data[0];
        if opcode != 2 { // opcode = 2 means system transfer
            return None;
        }

        let from_account = pubkeys[accounts_idx[0] as usize];
        let _to_account = pubkeys[accounts_idx[1] as usize];

        // ix_data[start..data_len] is transfer amount
        let start = data_len - 8; 
        let amount = u64::from_le_bytes(ix_data[start..data_len].try_into().unwrap());

        let bridge_tx_info = BridgeTxInfo{
            from: from_account,
            to: from_account, // from and to si same account
            amount,
            message_type: MessageType::Native,
        };
        let tx_info_hash = bridge_tx_info.double_hash_array();
        info!("dong: tx_info_hash {:?}", tx_info_hash);
        let sig = Signature::try_from(tx.signatures[0].clone()).unwrap();
        
        Some(BridgeTxRecord{
            slot: tx.slot,
            signature: sig.to_string(),
            tx_info_hash: tx_info_hash.into(),
            proof: "".to_string(),
            is_generated_proof: false,
            current_mt_root: vec![],
        })
    }
    pub fn insert_bridge_txs(&self, bridge_txs: Vec<BridgeTxRecord>) -> Result<u32, NodeError> {
        let repo = BridgeTxRepo{pool: Box::from(self.client_pool.to_owned())};

        let rows = repo.insert(bridge_txs)?;
        let count = rows.len() as u32;

        Ok(count)
    }

    pub fn brige_txs_hashes(&self, from_slot: i64, to_slot: i64) -> Result<Vec<Vec<u8>>, NodeError> {
        let repo = BridgeTxRepo{pool: Box::from(self.client_pool.to_owned())};

        repo.bridge_tx_hashes(from_slot, to_slot)
    }

    pub fn bridge_tx_range(&self, from_slot: i64, to_slot: i64) -> Result<Vec<BridgeTxRecord>, NodeError>{
        let repo = BridgeTxRepo{pool: Box::from(self.client_pool.to_owned())};

        let bridge_tx_rows = repo.range(from_slot, to_slot).unwrap();

        let bridge_tx_records = bridge_tx_rows.into_iter().map(BridgeTxRecord::from).collect();
        
        Ok(bridge_tx_records)

    }

    pub fn bridge_tx_update(&self, brige_tx_record: BridgeTxRecord) -> Result<BridgeTxRow, NodeError>{
        let repo = BridgeTxRepo{pool: Box::from(self.client_pool.to_owned())};

        let row = repo.update(brige_tx_record).unwrap();
        
        Ok(row)
    }
}


