use log::error;
use rocksdb::DB;
use std::i64;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

pub struct ChainRepo<'a> {
    pub db: &'a Arc<RwLock<DB>>,
}

impl<'a> ChainRepo<'a> {
    pub fn upsert(&self, value: i64) {
        let key = b"slot";
        let value_bytes = value.to_le_bytes(); // 序列化为字节数组 (little-endian)
        let db_write = self.db.write().unwrap(); // 获取写入锁
        db_write.put(key, value_bytes).unwrap();
    }

    pub fn show(&self) -> Option<i64> {
        let key = b"slot";
        let db_read = self.db.read().unwrap(); // 获取读取锁
        match db_read.get(key) {
            Ok(Some(value)) => {
                if value.len() == 8 { // 检查字节数组长度是否为 8
                    Some(i64::from_le_bytes(value.as_slice().try_into().unwrap())) // 反序列化为 i64
                } else {
                    None
                }
            }
            Ok(None) => None,
            Err(e) => {
                error!("get slot from rocksdb fail. err: {}", e);
                None
            }
        }
    }
}

