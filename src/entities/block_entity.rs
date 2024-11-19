diesel::table! {
    use diesel::sql_types::*;

    #[sql_name="block"]
    table_block(column_slot) {
        #[sql_name = "slot"]
        column_slot -> Int8,
    }
}