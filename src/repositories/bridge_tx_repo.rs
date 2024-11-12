use crate::common::node_error::NodeError;
use crate::entities::account_audit_entity::table_account_audit::column_write_version;
use crate::entities::bridge_transaction_entity::table_bridge_transaction::column_slot;
use crate::entities::bridge_transaction_entity::table_bridge_transaction::dsl::table_bridge_transaction;
use crate::models::bridge_transaction_model::{BridgeTxRecord, BridgeTxRow};
use crate::utils::store_util::{PgConnectionPool, PooledPgConnection};
use crate::utils::uuid_util::generate_uuid;
use diesel::prelude::*;
use diesel::RunQueryDsl;
use log::error;

pub struct BridgeTxRepo {
    pub pool: Box<PgConnectionPool>,
}

impl BridgeTxRepo {
    pub fn show(&self) -> Result<BridgeTxRow, NodeError> {
        let conn: &mut PooledPgConnection = &mut self.pool.get()?;

        let results = table_bridge_transaction
        .order(column_slot.desc())
        .limit(1)
        .load::<BridgeTxRow>(conn)
        .expect("Error loading chain");

        if results.is_empty() {
            return Err(
                NodeError::new(generate_uuid(), 
                "Couldn't find query last slot from database".to_string(),
                )
            );
        }

        let row = results[0].clone();
        Ok(row)
    }

    pub fn insert(&self, records: Vec<BridgeTxRecord>) -> Result<Vec<BridgeTxRow>, NodeError> {
        let conn: &mut PooledPgConnection = &mut self.pool.get()?;

        let rows = diesel::insert_into(table_bridge_transaction)
            .values(&records)
            .on_conflict_do_nothing()
            .get_results::<BridgeTxRow>(conn)
            .map_err(|e| {
                error!("Error insert bridge tx: {:?}", e);
                e
            })
            .expect("Error insert bridge tx");
        Ok(rows)
    }

    pub fn update(&self, record: BridgeTxRecord) -> Result<BridgeTxRow, NodeError> {
        let conn: &mut PooledPgConnection = &mut self.pool.get()?;
    
        let updated_row = diesel::update(table_bridge_transaction.filter(column_slot.eq(record.slot)))
            .set(&record) 
            .get_result::<BridgeTxRow>(conn)  
            .map_err(|e| {
                error!("Error updating bridge tx: {:?}", e);
                NodeError::new(generate_uuid(), format!("Error updating bridge tx: {:?}", e))
            })?;
    
        Ok(updated_row)
    }
    
    pub fn range(&self, from_slot: i64, to_slot: i64) -> Result<Vec<BridgeTxRow>, NodeError> {
        let conn: &mut PooledPgConnection = &mut self.pool.get()?;

        let rows = table_bridge_transaction
            .filter(column_slot.ge(from_slot).and(column_slot.le(to_slot)))
            .order(column_slot.asc())
            .load::<BridgeTxRow>(conn)
            .expect("Error loading bridge_tx");

        Ok(rows)
    }

    pub fn bridge_tx_hashes(&self, from_slot: i64, to_slot: i64) -> Result<Vec<Vec<u8>>, NodeError> {
        let bridge_txs = self.range(from_slot, to_slot).unwrap();
        let hashes = bridge_txs.into_iter().map(|t| {t.tx_info_hash}).collect();
        
        Ok(hashes)
    }
}