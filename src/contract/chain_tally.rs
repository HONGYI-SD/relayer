use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize)]
#[derive(Default)]
#[derive(Debug, Clone)]
#[derive(Eq, PartialEq)]
pub struct ChainTally {
    pub present_slot: u64,
}


impl ChainTally {
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
    use borsh::{BorshDeserialize, BorshSerialize};

    use crate::contract::chain_tally::ChainTally;

    fn prepare_tally() -> ChainTally {
        let slot: u64 = 11;

        let tally = ChainTally {
            present_slot: slot,
        };

        return tally;
    }


    //u64: 8 bytes
    #[test]
    fn test_basic() {
        let tally = prepare_tally();
        let mut data = Vec::new();
        tally.serialize(&mut data).unwrap();
        println!("serialized data {:?}", data);
        assert_eq!(ChainTally::size(), data.len());
    }

    #[test]
    fn test_convert() {
        let tally_actual = prepare_tally();
        let mut data = Vec::new();
        tally_actual.serialize(&mut data).unwrap();
        let tally_expect = ChainTally::deserialize(&mut &data[..]).unwrap();
        println!("deserialized data: {:?}", tally_expect);
        assert_eq!(tally_expect, tally_actual);
    }
}

