use diesel::prelude::*;
use diesel::row::NamedRow;
use diesel::RunQueryDsl;

use crate::common::node_error::NodeError;
use crate::entities::account_audit_entity::table_account_audit::{column_slot, column_write_version};
use crate::entities::account_audit_entity::table_account_audit::dsl::table_account_audit;
use crate::models::account_audit_row::AccountAuditRow;
use crate::utils::store_util::{PgConnectionPool, PooledPgConnection};
use crate::utils::uuid_util::generate_uuid;

pub struct AccountAuditRepo {
    pub pool: Box<PgConnectionPool>,
}

impl AccountAuditRepo {
    pub fn range(&self, from_slot: i64, to_slot: i64) -> Result<Vec<AccountAuditRow>, NodeError> {
        let conn: &mut PooledPgConnection = &mut self.pool.get()?;

        let rows = table_account_audit
            .filter(column_slot.ge(from_slot).and(column_slot.le(to_slot)))
            .order((column_slot.asc(), column_write_version.asc()))
            .load::<AccountAuditRow>(conn)
            .expect("Error loading account_audit");

        Ok(rows)
    }

    pub fn show(&self) -> Result<AccountAuditRow, NodeError> {
        let conn: &mut PooledPgConnection = &mut self.pool.get()?;

        let results = table_account_audit
            .order(column_slot.desc())
            .limit(1)
            .load::<AccountAuditRow>(conn)
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
}
