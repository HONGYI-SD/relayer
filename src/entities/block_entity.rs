diesel::table! {
    use diesel::sql_types::*;

    #[sql_name="block"]
    table_block(column_id) {
        #[sql_name = "id"]
        column_id -> Int8,

        #[sql_name = "slot"]
        column_slot -> Int8,
    }
}