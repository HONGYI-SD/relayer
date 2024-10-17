diesel::table! {
    use diesel::sql_types::*;

    #[sql_name="account_audit"]
    table_account_audit(column_id) {
        #[sql_name = "id"]
        column_id -> Int8,

        #[sql_name = "pubkey"]
        column_pubkey -> Bytea,

        #[sql_name = "owner"]
        column_owner -> Nullable<Bytea>,

        #[sql_name = "lamports"]
        column_lamports -> Int8,

        #[sql_name = "slot"]
        column_slot -> Int8,

        #[sql_name = "executable"]
        column_executable -> Bool,

        #[sql_name = "rent_epoch"]
        column_rent_epoch -> Int8,

        #[sql_name = "data"]
        column_data -> Nullable<Bytea>,

        #[sql_name = "write_version"]
        column_write_version -> Int8,

        #[sql_name = "txn_signature"]
        column_txn_signature -> Nullable<Bytea>,

        #[sql_name = "updated_on"]
        column_updated_on -> Timestamp,
    }
}

