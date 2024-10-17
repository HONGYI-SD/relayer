use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[derive(Default)]
#[derive(Debug, Clone)]
#[derive(Eq, PartialEq)]
#[derive(Ord, PartialOrd)]
pub struct WrapSlot {
    pub slot: u64,
}


impl WrapSlot {
    // slot: u64 needs 8 bytes
    pub fn size() -> usize {
        let slot_size: usize = 8;
        let total_size: usize = slot_size;
        return total_size;
    }

    //init
    pub fn init_size() -> usize {
        return Self::size();
    }

    //max
    pub fn total_size() -> usize {
        return Self::size();
    }
}


#[cfg(test)]
pub mod test {
    use crate::contract::wrap_slot::WrapSlot;

    fn prepare_wrap_slot() -> WrapSlot {
        let slot: u64 = 20;

        let wrap_slot = WrapSlot {
            slot,
        };

        return wrap_slot;
    }

    #[test]
    fn test_basic() {
        let wrap_slot = prepare_wrap_slot();
        let data = bincode::serialize(&wrap_slot).unwrap();
        assert_eq!(WrapSlot::size(), data.len());
    }

    #[test]
    fn test_convert() {
        let wrap_slot_actual = prepare_wrap_slot();
        let data = bincode::serialize(&wrap_slot_actual).unwrap();
        let wrap_slot_expect = bincode::deserialize::<WrapSlot>(&data[..]).unwrap();
        assert_eq!(wrap_slot_expect, wrap_slot_actual);
    }
}
