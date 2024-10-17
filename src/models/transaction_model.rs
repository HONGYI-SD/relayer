use crate::entities::sql_types::{DbLoadedMessageV0, DbTransactionMessage};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionRow {
    pub slot: i64,

    pub message_type: i16,

    pub legacy_message: Option<DbTransactionMessage>,

    pub v0_loaded_message: Option<DbLoadedMessageV0>,

    pub signatures: Vec<Vec<u8>>,
}
