use crate::common::node_configs::StoreConfiguration;
use crate::common::node_error::NodeError;
use crate::contract::chain_brief::ChainBrief;
use crate::models::account_audit_row::AccountAuditRow;
use crate::models::brief_model::convert_chain_briefs_to_brief_records;
use crate::models::transaction_model::TransactionRow;
use crate::repositories::account_audit_repo::AccountAuditRepo;
use crate::repositories::block_repo::BlockRepo;
use crate::repositories::brief_repo::BriefRepo;
use crate::repositories::chain_repo::ChainRepo;
use crate::repositories::transaction_repo::TransactionRepo;
use crate::smt::account_smt::{DatabaseStoreAccountSMT, MemoryStoreAccountSMT, SMTAccount};
use crate::smt::rocks_store::RocksStore;
use crate::utils::account_util::compute_ha;
use crate::utils::store_util::{create_one, create_pool, PgConnectionPool};
use crate::utils::uuid_util::generate_uuid;
use log::{error, info};
use postgres::Client;
use rocksdb::DB;
use solana_sdk::hash::Hash;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use std::collections::{BTreeMap, HashMap};
use std::path::Path;
use std::sync::{Arc, RwLock};

pub struct ExecuteService {
    client_pool: PgConnectionPool,
    client_one: Client,
    rocksdb: Arc<RwLock<DB>>,
    database_store_account_smt: Arc<RwLock<DatabaseStoreAccountSMT>>,
    memory_store_account_smt: Arc<RwLock<MemoryStoreAccountSMT>>,
    initial_slot: u64,
}

impl ExecuteService {
    pub fn new(config: &StoreConfiguration) -> Result<Self, NodeError> {
        let pool = create_pool(
            config.to_owned(),
            10,
        );

        let one = create_one(config.to_owned());

        let account_dir = Path::new("./fp-smt/account");
        let account_db = DB::open_default(account_dir).unwrap();
        let rocksdb_store = RocksStore::new(account_db);
        let slot_dir = Path::new("./fp-smt/slot");
        let slot_db = DB::open_default(slot_dir).unwrap();
        let rocksdb = Arc::new(RwLock::new(slot_db));
        let database_store_account_sparse_merkle_tree = DatabaseStoreAccountSMT::new_with_store(rocksdb_store).unwrap();
        let db_smt = Arc::new(RwLock::new(database_store_account_sparse_merkle_tree));
        let memory_store_account_sparse_merkle_tree = MemoryStoreAccountSMT::new_with_store(Default::default()).unwrap();
        let mm_smt = Arc::new(RwLock::new(memory_store_account_sparse_merkle_tree));

        info!("Created PostgresClient.");

        Ok(Self {
            client_pool: pool,
            client_one: one,
            rocksdb,
            database_store_account_smt: db_smt,
            memory_store_account_smt: mm_smt,
            // The slot 0 and slot 1 are initial of blockchain, skip them.
            initial_slot: 2,
        })
    }

    pub fn check_smt(&mut self) -> Result<bool, NodeError> {
        if self.database_store_account_smt.read().unwrap().is_empty() {
            info!("database smt is empty.");
            return Ok(true);
        }

        // 获取最后处理的区块高度
        let last_slot = self.get_last_slot().unwrap_or(self.initial_slot as i64);

        let start_slot = self.initial_slot as i64;
        let end_slot = last_slot;

        let account_audits = self.get_account_audits(start_slot, end_slot).unwrap();

        let kvs = self.convert(account_audits);
        self.memory_store_account_smt.write().unwrap().update_all(kvs).unwrap();

        let database_store_account_smt_root = Hash::new(self.database_store_account_smt.read().unwrap().root().as_slice().clone()).to_string();
        let memory_store_account_smt_root = Hash::new(self.memory_store_account_smt.read().unwrap().root().as_slice().clone()).to_string();

        let flag = database_store_account_smt_root == memory_store_account_smt_root;

        if flag {
            info!("database smt root and memory smt root is same. database smt root: {:?}, memory smt root: {:?}",
                database_store_account_smt_root,memory_store_account_smt_root);
        } else {
            error!("database smt root and memory smt root is different! database smt root: {:?}, memory smt root: {:?}",
                database_store_account_smt_root,memory_store_account_smt_root);
        }

        Ok(flag)
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

    pub fn generate_briefs(&mut self, start_slot: i64, end_slot: i64) -> Result<Vec<ChainBrief>, NodeError> {
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

        let mut slot_to_transactions: BTreeMap<i64, Vec<TransactionRow>> = BTreeMap::new();
        for transaction in transactions.clone() {
            slot_to_transactions.entry(transaction.slot).or_insert_with(Vec::new).push(transaction);
        }

        let account_audits = self.get_account_audits(start_slot, end_slot)?;

        let mut slot_to_account_audits: BTreeMap<i64, Vec<AccountAuditRow>> = BTreeMap::new();
        for account_audit in account_audits.clone() {
            slot_to_account_audits.entry(account_audit.slot).or_insert_with(Vec::new).push(account_audit);
        }

        let mut slot_to_root_hash: HashMap<i64, String> = HashMap::new();
        for (slot, account_audits) in slot_to_account_audits.clone() {
            let kvs = self.convert(account_audits);
            self.memory_store_account_smt.write().unwrap().update_all(kvs).unwrap();
            let root_hash =
                Hash::new(self.memory_store_account_smt.read().unwrap().root().as_slice().clone()).to_string();
            slot_to_root_hash.insert(slot, root_hash);
        }

        info!("start_slot: {:?} end_slot: {:?}", start_slot, end_slot);


        let mut slot_to_hash_account: HashMap<i64, String> = HashMap::new();
        let mut slot_to_transaction_number: HashMap<i64, u32> = HashMap::new();
        for (slot, transactions) in slot_to_transactions.clone() {
            let account_audits = slot_to_account_audits.get(&slot.clone()).unwrap();
            let mut ha = Hash::default();
            for r in transactions.clone() {
                let msg = r.legacy_message.unwrap();
                let signatures = r.signatures;
                let pks: Vec<Pubkey> = msg.account_keys.iter().map(|ak| Pubkey::try_from(ak.as_slice()).unwrap()).collect();

                // Found all related (modified) accounts from account_audit
                let modified_accounts: Vec<AccountAuditRow> = account_audits
                    .iter()
                    .filter(|a| pks.contains(&Pubkey::try_from(a.pubkey.as_slice()).unwrap()) && a.txn_signature.is_some())
                    .cloned()
                    .collect();

                ha = compute_ha(
                    &Signature::try_from(signatures[0].as_slice()).unwrap(),
                    &modified_accounts
                        .iter()
                        .map(|a| a.to_smt_account())
                        .collect::<Vec<SMTAccount>>(),
                    &ha,
                );
            }

            let hash_account = ha.to_string();
            slot_to_hash_account.insert(slot, hash_account);
            let transaction_number = transactions.len() as u32;
            slot_to_transaction_number.insert(slot, transaction_number);
        }

        // 根据检查结果组装 briefs
        let mut briefs: Vec<ChainBrief> = Vec::new();

        for slot in start_slot..=end_slot {
            if let (Some(hash_account), Some(transaction_number), Some(root_hash)) = (
                slot_to_hash_account.get(&slot),
                slot_to_transaction_number.get(&slot),
                slot_to_root_hash.get(&slot),
            ) {
                briefs.push(ChainBrief {
                    slot: slot as u64,
                    hash_account: hash_account.clone(),
                    transaction_number: *transaction_number,
                    root_hash: root_hash.clone(),
                });
            }
        }

        self.insert_briefs(briefs.clone()).expect("insert briefs failed");

        let kvs = self.convert(account_audits.clone());
        self.database_store_account_smt.write().unwrap().update_all(kvs).unwrap();
        self.update_last_slot(end_slot);

        Ok(briefs)
    }

    fn convert(&self, account_audits: Vec<AccountAuditRow>) -> Vec<(sparse_merkle_tree::H256, SMTAccount)> {
        let mut kvs: Vec<(sparse_merkle_tree::H256, SMTAccount)> = Vec::new();
        account_audits.iter().for_each(|aa| {
            if aa.txn_signature.is_some() {
                let raw_acct = aa.to_smt_account();
                let key_hash = raw_acct.smt_key();
                kvs.push((key_hash, raw_acct));
            }
        });

        kvs
    }
}


