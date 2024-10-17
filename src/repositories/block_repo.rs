use crate::common::node_error::NodeError;
use crate::models::block_model::BlockRow;
use postgres::Client;

pub struct BlockRepo<'a> {
    pub one: &'a mut Client,
}

impl<'a> BlockRepo<'a> {
    pub fn show(&mut self) -> Result<BlockRow, NodeError> {
        let conn = &mut self.one;

        let max_slot_stmt = "SELECT MAX(slot) FROM block";
        let tx_results = conn.query(max_slot_stmt, &[]).unwrap();
        let rows: Vec<BlockRow> = tx_results.into_iter().map(|r| {
            BlockRow {
                slot: r.get(0),
            }
        }).collect();

        let row: BlockRow = rows.first().unwrap().clone();

        Ok(row)
    }
}
