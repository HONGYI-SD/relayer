diesel::table! {
    use diesel::sql_types::*;

    #[sql_name="bridge_transaction"]
    table_bridge_transaction(column_id) {
        #[sql_name = "id"]
        column_id -> Int8,

        #[sql_name = "slot"]
        column_slot -> Int8,

        #[sql_name = "signature"]
        column_signature -> VarChar,

        #[sql_name = "tx_info_hash"]
        column_tx_info_hash -> Bytea,

        #[sql_name = "proof"]
        column_proof -> VarChar,

        #[sql_name = "updated_on"]
        column_updated_on -> Timestamp,
    }
}
