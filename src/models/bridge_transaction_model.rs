use crate::entities::bridge_transaction_entity::table_bridge_transaction;
use crate::models::transaction_model::TransactionRow;
use dd_merkle_tree::HashingAlgorithm;
use diesel::Selectable;
use diesel::{Insertable, Queryable, AsChangeset};
use lombok::{Getter, Setter};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

#[derive(Debug, Clone, Deserialize, Serialize, Setter, Getter)]
#[serde(rename_all(serialize = "snake_case", deserialize = "snake_case"))]
pub struct BridgeTxData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_info_hash: Option<Vec<u8>>,

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

    #[diesel(sql_type = Bytea)]
    #[diesel(column_name = column_tx_info_hash)]
    pub tx_info_hash: Vec<u8>,

    #[diesel(sql_type = Varchar)]
    #[diesel(column_name = column_proof)]
    pub proof: String,

    #[diesel(sql_type = Bool)]
    #[diesel(column_name = column_is_generated_proof)]
    pub is_generated_proof: bool,

    #[diesel(sql_type = Bytea)]
    #[diesel(column_name = column_current_mt_root)]
    pub current_mt_root: Vec<u8>,

    #[diesel(sql_type = Int8)]
    #[diesel(column_name = column_root_program_slot)]
    pub root_program_slot: i64,

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

    #[diesel(sql_type = Bytea)]
    #[diesel(column_name = column_tx_info_hash)]
    pub tx_info_hash: Vec<u8>,

    #[diesel(sql_type = Varchar)]
    #[diesel(column_name = column_proof)]
    pub proof: String,

    #[diesel(sql_type = Bool)]
    #[diesel(column_name = column_is_generated_proof)]
    pub is_generated_proof: bool,

    #[diesel(sql_type = Int8)]
    #[diesel(column_name = column_root_program_slot)]
    pub root_program_slot: i64,

    #[diesel(sql_type = Bytea)]
    #[diesel(column_name = column_current_mt_root)]
    pub current_mt_root: Vec<u8>,
}

// impl From<&TransactionRow> for BridgeTxRecord {
//     fn from(tr: &TransactionRow) -> Self {
//         BridgeTxRecord { 
//             slot: tr.slot, 
//             signature: "todo signature".to_string(),
//             tx_info_hash: "todo tx hash".as_bytes().to_vec(),
//             proof: "tx proof".to_string(),
//             is_generated_proof: false,
//             current_mt_root:
//         }
//     }
// }

impl From<BridgeTxRow> for BridgeTxRecord {
    fn from(btr: BridgeTxRow) -> Self {
        BridgeTxRecord { 
            slot: btr.slot, 
            signature: btr.signature,
            tx_info_hash: btr.tx_info_hash,
            proof: btr.proof,
            is_generated_proof: btr.is_generated_proof,
            root_program_slot: 0 as i64,
            current_mt_root: btr.current_mt_root
        }
    }
    
}

#[derive(Debug)]
pub struct BridgeTxInfo {
    pub from: Pubkey,
    pub to: Pubkey,
    pub amount: u64,
    pub bridge_tx_index: u64,
    pub message_type: MessageType,
}

impl BridgeTxInfo {
    pub fn new(from: Pubkey, to: Pubkey, amount: u64, message_type: MessageType) -> Self {
        Self {
            from,
            to,
            amount,
            bridge_tx_index: 0,
            message_type,
        }
    }
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.from.to_bytes());
        bytes.extend_from_slice(&self.to.to_bytes());
        bytes.extend_from_slice(&self.amount.to_le_bytes());
        bytes.extend_from_slice(&self.bridge_tx_index.to_le_bytes());
        bytes.extend_from_slice(&self.message_type.to_bytes());
        bytes
    }

    pub fn double_hash(&self) -> Vec<u8> {
        let m = &self.to_bytes();
        HashingAlgorithm::Sha256d.double_hash(m, 32 as usize)
    }

    pub fn double_hash_array(&self) -> [u8; 32] {
        let m = self.double_hash();
        assert!(m.len() == 32);
        let mut array = [0u8; 32];
        array.copy_from_slice(&m);
        array
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum MessageType {
    Native,
    Token,
    NFT,
}

impl MessageType {
    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        bincode::deserialize(bytes).unwrap()
    }
}