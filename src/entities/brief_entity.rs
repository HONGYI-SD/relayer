diesel::table! {
    use diesel::sql_types::*;

    #[sql_name="brief"]
    table_brief(column_id) {
        #[sql_name = "id"]
        column_id -> Int8,

        #[sql_name = "slot"]
        column_slot -> Int8,

        #[sql_name = "root_hash"]
        column_root_hash -> Varchar,

        #[sql_name = "hash_account"]
        column_hash_account -> Varchar,

        #[sql_name = "transaction_number"]
        column_transaction_number -> Int4,

        #[sql_name = "updated_on"]
        column_updated_on -> Timestamp,
    }
}
