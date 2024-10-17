use std::marker::PhantomData;

use rocksdb::*;
use sparse_merkle_tree::{
    error::Error,
    traits::{StoreReadOps, StoreWriteOps, Value},
    BranchKey, BranchNode, H256,
};

use super::serde::{branch_key_to_vec, branch_node_to_vec, slice_to_branch_node};

/// A SMT `Store` implementation backed by a RocksDB database, using the default column family.
pub struct RocksStore<W> {
    // The RocksDB database which stores the data, can be a `DB` / `OptimisticTransactionDB` / `Snapshot` etc.
    inner: DB,
    // A generic write options, can be a `WriteOptions` / `()` etc.
    write_options: PhantomData<W>,
}

impl<W> RocksStore<W> {
    pub fn new(db: DB) -> Self {
        RocksStore {
            inner: db,
            write_options: PhantomData,
        }
    }
}

impl<V, W> StoreReadOps<V> for RocksStore<W>
where
    V: Value + std::convert::From<std::vec::Vec<u8>>,
{
    fn get_branch(&self, branch_key: &BranchKey) -> Result<Option<BranchNode>, Error> {
        self.inner
            .get(&branch_key_to_vec(branch_key))
            .map(|s| s.map(|v| slice_to_branch_node(&v)))
            .map_err(|e| Error::Store(e.to_string()))
    }

    fn get_leaf(&self, leaf_key: &H256) -> Result<Option<V>, Error> {
        self.inner
            .get(leaf_key.as_slice())
            .map(|s| s.map(|v| v.into()))
            .map_err(|e| Error::Store(e.to_string()))
    }
}

impl<V, W> StoreWriteOps<V> for RocksStore<W>
where
    V: Value + std::convert::From<std::vec::Vec<u8>> + Into<Vec<u8>>,
{
    fn insert_branch(&mut self, node_key: BranchKey, branch: BranchNode) -> Result<(), Error> {
        self.inner
            .put(&branch_key_to_vec(&node_key), &branch_node_to_vec(&branch))
            .map_err(|e| Error::Store(e.to_string()))
    }

    fn insert_leaf(&mut self, leaf_key: H256, leaf: V) -> Result<(), Error> {
        let v: Vec<u8> = leaf.into();
        self.inner
            .put(leaf_key.as_slice(), v)
            .map_err(|e| Error::Store(e.to_string()))
    }

    fn remove_branch(&mut self, node_key: &BranchKey) -> Result<(), Error> {
        self.inner
            .delete(&branch_key_to_vec(node_key))
            .map_err(|e| Error::Store(e.to_string()))
    }

    fn remove_leaf(&mut self, leaf_key: &H256) -> Result<(), Error> {
        self.inner
            .delete(leaf_key.as_slice())
            .map_err(|e| Error::Store(e.to_string()))
    }
}

#[allow(unused_imports)]
#[allow(dead_code)]
mod tests {
    use rocksdb::{OptimisticTransactionDB, Options, DB};
    use sparse_merkle_tree::{
        blake2b::Blake2bHasher, default_store::DefaultStore as SMTDefaultStore, traits::Value,
        SparseMerkleTree, H256,
    };

    use super::RocksStore;
    use blake2b_rs::{Blake2b, Blake2bBuilder};
    use std::path::Path;

    #[allow(dead_code)]
    pub fn new_blake2b() -> Blake2b {
        Blake2bBuilder::new(32).personal(b"SMT").build()
    }

    #[derive(Default, Clone)]
    pub struct Word(String);

    impl Value for Word {
        fn to_h256(&self) -> H256 {
            if self.0.is_empty() {
                return H256::zero();
            }
            let mut buf = [0u8; 32];
            let mut hasher = new_blake2b();
            hasher.update(self.0.as_bytes());
            hasher.finalize(&mut buf);
            buf.into()
        }

        fn zero() -> Self {
            Default::default()
        }
    }

    impl Into<Vec<u8>> for Word {
        fn into(self) -> Vec<u8> {
            self.0.into_bytes()
        }
    }

    // impl AsRef<[u8]> for Word {
    //     fn as_ref(&self) -> &[u8] {
    //         self.0.as_bytes()
    //     }
    // }

    impl From<Vec<u8>> for Word {
        fn from(vec: Vec<u8>) -> Self {
            Word(String::from_utf8(vec).expect("stored value is utf8"))
        }
    }

    type MemoryStoreSMT = SparseMerkleTree<Blake2bHasher, Word, SMTDefaultStore<Word>>;
    type RocksStoreSMT = SparseMerkleTree<Blake2bHasher, Word, RocksStore<Word>>;

    #[test]
    fn test_rocks_store_functions() {
        // cargo test --package smt --lib -- rocks_store::tests::test_rocks_store_functions --exact --nocapture
        let kvs = "The quick brown fox jumps over the lazy dog"
            .split_whitespace()
            .enumerate()
            .map(|(i, word)| {
                let mut buf = [0u8; 32];
                let mut hasher = new_blake2b();
                hasher.update(&(i as u32).to_le_bytes());
                hasher.finalize(&mut buf);
                (buf.into(), Word(word.to_string()))
            })
            .collect::<Vec<(H256, Word)>>();

        // generate a merkle tree with a memory store
        let (root1, proof1) = {
            let mut memory_store_smt = MemoryStoreSMT::new_with_store(Default::default()).unwrap();
            for (key, value) in kvs.iter() {
                memory_store_smt.update(key.clone(), value.clone()).unwrap();
            }
            let root = memory_store_smt.root().clone();
            let proof = memory_store_smt
                .merkle_proof(vec![kvs[0].0.clone()])
                .unwrap();
            (root, proof)
        };

        // generate a merkle tree with a rocksdb store
        let dir = Path::new("./rocks-test-default");
        let (root2, proof2) = {
            let db = DB::open_default(dir).unwrap();
            let rocksdb_store = RocksStore::new(db);
            let mut rocksdb_store_smt = RocksStoreSMT::new_with_store(rocksdb_store).unwrap();
            // Write db
            for (key, value) in kvs.iter() {
                rocksdb_store_smt
                    .update(key.clone(), value.clone())
                    .unwrap();
            }
            let root = rocksdb_store_smt.root().clone();
            println!("wrote root: {:?}", root);
            let proof = rocksdb_store_smt
                .merkle_proof(vec![kvs[0].0.clone()])
                .unwrap();
            (root, proof)
        };
        assert_eq!(root1, root2);
        assert_eq!(proof1, proof2);

        // Read db
        {
            let db = DB::open_default(dir).unwrap();
            let rocksdb_store = RocksStore::new(db);

            // parse db as smt
            let rocksdb_store_smt = RocksStoreSMT::new_with_store(rocksdb_store).unwrap();

            let root = rocksdb_store_smt.root().clone();
            println!("read root: {:?}", root);

            for (key, value) in kvs.iter() {
                let stored_value = rocksdb_store_smt.get(key).unwrap();
                println!(
                    "value: {}, stored_value: {}",
                    value.clone().0,
                    stored_value.0
                );
            }

            assert_eq!(root2, root);
        }
    }
}
