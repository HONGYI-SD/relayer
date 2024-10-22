use crate::entities::bridge_transaction_entity::table_bridge_transaction;
use crate::models::transaction_model::TransactionRow;
use diesel::Selectable;
use diesel::{Insertable, Queryable, AsChangeset};
use lombok::{Getter, Setter};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Setter, Getter)]
#[serde(rename_all(serialize = "snake_case", deserialize = "snake_case"))]
pub struct BridgeTxData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_hash: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof: Option<String>,
}

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = table_bridge_transaction)]
pub struct BridgeTxRow {
    #[diesel(sql_type = Int8)]
    #[diesel(column_name = column_id)]
    pub id: i64,

    #[diesel(sql_type = Int8)]
    #[diesel(column_name = column_slot)]
    pub slot: i64,

    #[diesel(sql_type = Varchar)]
    #[diesel(column_name = column_signature)]
    pub signature: String,

    #[diesel(sql_type = Varchar)]
    #[diesel(column_name = column_tx_hash)]
    pub tx_hash: String,

    #[diesel(sql_type = Varchar)]
    #[diesel(column_name = column_proof)]
    pub proof: String,

    #[diesel(sql_type = Timestamp)]
    #[diesel(column_name = column_updated_on)]
    pub updated_on: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Insertable, Serialize, AsChangeset, Deserialize)]
#[diesel(table_name = table_bridge_transaction)]
pub struct BridgeTxRecord {
    #[diesel(sql_type = Int8)]
    #[diesel(column_name = column_slot)]
    pub slot: i64,

    #[diesel(sql_type = Varchar)]
    #[diesel(column_name = column_signature)]
    pub signature: String,

    #[diesel(sql_type = Varchar)]
    #[diesel(column_name = column_tx_hash)]
    pub tx_hash: String,

    #[diesel(sql_type = Varchar)]
    #[diesel(column_name = column_proof)]
    pub proof: String,
}

impl From<TransactionRow> for BridgeTxRecord {
    fn from(tr: TransactionRow) -> Self {
        BridgeTxRecord { 
            slot: tr.slot, 
            signature: "todo signature".to_string(), //todo tr.signatures[0].as_slice(), 
            tx_hash: "todo tx hash".to_string(),  //todo, compute bridge tx hash
            proof: "tx proof".to_string() // todo
        }
    }
}