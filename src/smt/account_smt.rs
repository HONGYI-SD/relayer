use blake2b_rs::Blake2bBuilder;
use solana_sdk::{account::Account, pubkey::Pubkey};
use sparse_merkle_tree::{blake2b::Blake2bHasher, H256, SparseMerkleTree, traits::Value};
use sparse_merkle_tree::default_store::DefaultStore;

use crate::smt::rocks_store::RocksStore;

pub const PUBKEY_BYTES: usize = 32;

#[derive(Default, Clone, Debug)]
pub struct SMTAccount {
    pub pubkey: Pubkey,
    pub lamports: i64,
    pub owner: Pubkey,
    pub executable: bool,
    pub rent_epoch: i64,
    pub data: Vec<u8>,
}

impl SMTAccount {
    pub fn smt_key(&self) -> H256 {
        let mut buf = [0u8; 32];
        let mut blake2b = Blake2bBuilder::new(32).build();
        blake2b.update(&self.pubkey.as_ref());
        blake2b.finalize(&mut buf);
        buf.into()
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.extend_from_slice(&self.pubkey.as_ref());
        bytes.extend_from_slice(&self.lamports.to_le_bytes());
        bytes.extend_from_slice(&self.owner.as_ref());
        bytes.extend_from_slice(&[self.executable as u8]);
        bytes.extend_from_slice(&self.rent_epoch.to_le_bytes());
        bytes.extend_from_slice(&self.data);
        bytes
    }

    pub fn to_normal_account(&self) -> Account {
        Account {
            lamports: self.lamports as u64,
            data: self.data.clone(),
            owner: self.owner,
            executable: self.executable,
            rent_epoch: self.rent_epoch as u64,
        }
    }
}

impl Value for SMTAccount {
    fn to_h256(&self) -> H256 {
        let mut buf = [0u8; 32];
        let mut blake2b = Blake2bBuilder::new(32).build();
        blake2b.update(&self.to_vec());
        blake2b.finalize(&mut buf);
        buf.into()
    }

    fn zero() -> Self {
        Default::default()
    }
}

impl From<Vec<u8>> for SMTAccount {
    fn from(bytes: Vec<u8>) -> Self {
        let mut index = 0;
        let pubkey = Pubkey::try_from(&bytes[0..PUBKEY_BYTES]).unwrap();
        index = PUBKEY_BYTES;

        let lamports_bytes: [u8; 8] =
            bytes[index..index + 8]
                .to_vec()
                .try_into()
                .unwrap_or_else(|v: Vec<u8>| {
                    panic!("Expected a Vec of length {} but it was {}", 8, v.len())
                });
        let lamports = i64::from_le_bytes(lamports_bytes);
        index += 8;

        let owner = Pubkey::try_from(&bytes[index..index + PUBKEY_BYTES]).unwrap();
        index += PUBKEY_BYTES;

        let executable = bytes[index] != 0;
        index += 1;

        let rent_epoch_bytes: [u8; 8] =
            bytes[index..index + 8]
                .to_vec()
                .try_into()
                .unwrap_or_else(|v: Vec<u8>| {
                    panic!("Expected a Vec of length {} but it was {}", 8, v.len())
                });
        let rent_epoch = i64::from_le_bytes(rent_epoch_bytes);
        index += 8;

        let data = bytes[index..].to_vec();
        Self {
            pubkey,
            lamports,
            owner,
            executable,
            rent_epoch,
            data,
        }
    }
}

impl Into<Vec<u8>> for SMTAccount {
    fn into(self) -> Vec<u8> {
        self.to_vec()
    }
}

pub type DatabaseStoreAccountSMT = SparseMerkleTree<Blake2bHasher, SMTAccount, RocksStore<SMTAccount>>;
pub type MemoryStoreAccountSMT = SparseMerkleTree<Blake2bHasher, SMTAccount, DefaultStore<SMTAccount>>;
