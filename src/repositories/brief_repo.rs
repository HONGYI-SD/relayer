use crate::common::node_error::NodeError;
use crate::entities::brief_entity::table_brief::column_slot;
use crate::entities::brief_entity::table_brief::dsl::table_brief;
use crate::models::brief_model::{BriefRecord, BriefRow};
use crate::utils::store_util::{PgConnectionPool, PooledPgConnection};
use crate::utils::uuid_util::generate_uuid;
use diesel::prelude::*;
use diesel::row::NamedRow;
use diesel::RunQueryDsl;
use log::error;

pub struct BriefRepo {
    pub pool: Box<PgConnectionPool>,
}

impl BriefRepo {
    pub fn show(&self) -> Result<BriefRow, NodeError> {
        let conn: &mut PooledPgConnection = &mut self.pool.get()?;

        let results = table_brief
            .order(column_slot.desc())
            .limit(1)
            .load::<BriefRow>(conn)
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

    pub fn insert(&self, records: Vec<BriefRecord>) -> Result<Vec<BriefRow>, NodeError> {
        let conn: &mut PooledPgConnection = &mut self.pool.get()?;

        let rows = diesel::insert_into(table_brief)
            .values(&records)
            .on_conflict_do_nothing()
            .get_results::<BriefRow>(conn)
            .map_err(|e| {
                error!("Error insert brief: {:?}", e);
                e
            })
            .expect("Error insert brief");

        Ok(rows)
    }
}
