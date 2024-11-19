use diesel::{Queryable, Selectable};
use lombok::Getter;
use serde_derive::{Deserialize, Serialize};

use crate::entities::block_entity::table_block;

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize, Getter)]
#[diesel(table_name = table_block)]
pub struct BlockRow {
    #[diesel(sql_type = Int8)]
    #[diesel(column_name = column_id)]
    pub id: i64,

    #[diesel(sql_type = Int8)]
    #[diesel(column_name = column_slot)]
    pub slot: i64,
}
