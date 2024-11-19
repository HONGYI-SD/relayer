use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};

use crate::common::node_error::NodeError;
use crate::entities::block_entity::table_block::column_slot;
use crate::entities::block_entity::table_block::dsl::table_block;
use crate::models::block_model::BlockRow;
use crate::utils::store_util::{PgConnectionPool, PooledPgConnection};
use crate::utils::uuid_util::generate_uuid;

pub struct BlockRepo {
    pub pool: Box<PgConnectionPool>,
}

impl BlockRepo {
    pub fn show(&mut self) -> Result<BlockRow, NodeError> {
        let conn: &mut PooledPgConnection = &mut self.pool.get()?;
        
        let results = table_block
        .order(column_slot.desc())
        .limit(1)
        .load::<BlockRow>(conn)
        .expect("Error loading block");

        if results.is_empty() {
            return Err(
                NodeError::new(generate_uuid(),
                "".to_string(),
             )
            );
        }
        
        let row = results[0].clone();
        Ok(row)
    }
}
