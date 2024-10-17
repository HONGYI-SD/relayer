use diesel::Queryable;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use crate::smt::account_smt::SMTAccount;

#[derive(Debug, Clone, Queryable, Serialize, Deserialize)]
#[diesel(table_name = table_account_audit)]
pub struct AccountAuditRow {
    #[diesel(sql_type = Int8)]
    #[diesel(column_name = column_id)]
    pub id: i64,

    #[diesel(sql_type = Bytea)]
    #[diesel(column_name = column_pubkey)]
    pub pubkey: Vec<u8>,

    #[diesel(sql_type = Nullable < Bytea >)]
    #[diesel(column_name = column_owner)]
    pub owner: Option<Vec<u8>>,

    #[diesel(sql_type = Int8)]
    #[diesel(column_name = column_lamports)]
    pub lamports: i64,

    #[diesel(sql_type = Int8)]
    #[diesel(column_name = column_slot)]
    pub slot: i64,

    #[diesel(sql_type = Bool)]
    #[diesel(column_name = column_executable)]
    pub executable: bool,

    #[diesel(sql_type = Int8)]
    #[diesel(column_name = column_rent_epoch)]
    pub rent_epoch: i64,

    #[diesel(sql_type = Nullable < Bytea >)]
    #[diesel(column_name = column_data)]
    pub data: Option<Vec<u8>>,

    #[diesel(sql_type = Int8)]
    #[diesel(column_name = column_write_version)]
    pub write_version: i64,

    #[diesel(sql_type = Nullable < Bytea >)]
    #[diesel(column_name = column_txn_signature)]
    pub txn_signature: Option<Vec<u8>>,

    #[diesel(sql_type = Timestamp)]
    #[diesel(column_name = column_updated_on)]
    pub updated_on: chrono::NaiveDateTime,
}


impl AccountAuditRow {
    pub fn to_smt_account(&self) -> SMTAccount {
        SMTAccount {
            pubkey: Pubkey::try_from(self.pubkey.as_slice()).unwrap(),
            lamports: self.lamports,
            owner: Pubkey::try_from(self.owner.clone().unwrap().as_slice()).unwrap(),
            executable: self.executable,
            rent_epoch: self.rent_epoch,
            data: self.data.clone().unwrap(),
        }
    }
}
