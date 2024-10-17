use crate::contract::chain_brief::ChainBrief;
use crate::entities::brief_entity::table_brief;
use diesel::Selectable;
use diesel::{Insertable, Queryable};
use lombok::{Getter, Setter};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Setter, Getter)]
#[serde(rename_all(serialize = "snake_case", deserialize = "snake_case"))]
pub struct BriefData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_hash: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash_account: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_number: Option<i32>,
}

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = table_brief)]
pub struct BriefRow {
    #[diesel(sql_type = Int8)]
    #[diesel(column_name = column_id)]
    pub id: i64,

    #[diesel(sql_type = Int8)]
    #[diesel(column_name = column_slot)]
    pub slot: i64,

    #[diesel(sql_type = Varchar)]
    #[diesel(column_name = column_root_hash)]
    pub root_hash: String,

    #[diesel(sql_type = Varchar)]
    #[diesel(column_name = column_hash_account)]
    pub hash_account: String,

    #[diesel(sql_type = Int4)]
    #[diesel(column_name = column_transaction_number)]
    pub transaction_number: i32,

    #[diesel(sql_type = Timestamp)]
    #[diesel(column_name = column_updated_on)]
    pub updated_on: chrono::NaiveDateTime,
}


#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = table_brief)]
pub struct BriefRecord {
    #[diesel(sql_type = Int8)]
    #[diesel(column_name = column_slot)]
    pub column_slot: i64,

    #[diesel(sql_type = Varchar)]
    #[diesel(column_name = column_root_hash)]
    pub column_root_hash: String,

    #[diesel(sql_type = Varchar)]
    #[diesel(column_name = column_hash_account)]
    pub column_hash_account: String,

    #[diesel(sql_type = Int4)]
    #[diesel(column_name = column_transaction_number)]
    pub column_transaction_number: i32,
}


impl From<ChainBrief> for BriefRecord {
    fn from(chain_brief: ChainBrief) -> Self {
        BriefRecord {
            column_slot: chain_brief.slot as i64,
            column_root_hash: chain_brief.root_hash,
            column_hash_account: chain_brief.hash_account,
            column_transaction_number: chain_brief.transaction_number as i32,
        }
    }
}

pub fn convert_chain_briefs_to_brief_records(chain_briefs: Vec<ChainBrief>) -> Vec<BriefRecord> {
    chain_briefs.into_iter().map(BriefRecord::from).collect()
}
