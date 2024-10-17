use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize)]
#[derive(Default)]
#[derive(Debug, Clone)]
#[derive(Eq, PartialEq)]
pub struct ChainBrief {
    pub slot: u64,
    pub root_hash: String,
    pub hash_account: String,
    pub transaction_number: u32,
}


impl ChainBrief {
    // slot: u64 needs 8 bytes
    // root_hash: String needs 44+4 bytes
    // hash_account: String needs 44+4 bytes
    // transaction_number: u32 needs 4 bytes
    pub fn size() -> usize {
        let slot_size: usize = 8;
        let root_hash_size: usize = 44 + 4;
        let hash_account_size: usize = 44 + 4;
        let transaction_number_size: usize = 4;
        let total_size: usize = slot_size + root_hash_size + hash_account_size + transaction_number_size;

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

    use crate::contract::chain_brief::ChainBrief;

    fn prepare_brief() -> ChainBrief {
        let slot: u64 = 11;
        let root_hash: String = "CodF5mXgscuEnvfYHVfKwGPosffRucTtAm4BQpyyjL8U".to_string();
        let hash_account: String = "BsVLhVaLeZpVnwxWqUF4bnpfLurcKYLq576Xg34RX3yQ".to_string();
        let transaction_number: u32 = 5;

        let brief = ChainBrief {
            slot,
            root_hash,
            hash_account,
            transaction_number,
        };

        return brief;
    }


    //u64: 8 bytes
    #[test]
    fn test_basic() {
        let brief = prepare_brief();
        let mut data = Vec::new();
        brief.serialize(&mut data).unwrap();
        println!("serialized data {:?}", data);
        assert_eq!(ChainBrief::size(), data.len());
    }

    #[test]
    fn test_convert() {
        let brief_actual = prepare_brief();
        let mut data = Vec::new();
        brief_actual.serialize(&mut data).unwrap();
        let brief_expect = ChainBrief::deserialize(&mut &data[..]).unwrap();
        println!("deserialized data: {:?}", brief_expect);
        assert_eq!(brief_expect, brief_actual);
    }
}

