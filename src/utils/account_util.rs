use blake2b_rs::Blake2bBuilder;
use solana_sdk::account::Account;
use solana_sdk::hash::Hash;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use crate::smt::account_smt::SMTAccount;

pub fn account_to_vec(pk: &Pubkey, account: &Account) -> Vec<u8> {
    let mut bytes = vec![];
    bytes.extend_from_slice(&pk.as_ref());
    bytes.extend_from_slice(&account.lamports.to_le_bytes());
    bytes.extend_from_slice(&account.owner.as_ref());
    bytes.extend_from_slice(&[account.executable as u8]);
    bytes.extend_from_slice(&account.rent_epoch.to_le_bytes());
    bytes.extend_from_slice(&account.data);
    bytes
}

pub fn compute_ha(tx_id: &Signature, accounts: &Vec<SMTAccount>, old_ha: &Hash) -> Hash {
    // println!("compute_ha: {:?}", accounts);
    let mut buf = [0u8; 32];
    let mut blake2b = Blake2bBuilder::new(32).build();
    blake2b.update(&tx_id.as_ref());
    let mut accounts = accounts.clone();
    accounts.sort_by(|a, b| a.pubkey.cmp(&b.pubkey));
    accounts.iter().for_each(|a| {
        // println!("sorted account: {}", a.pubkey);
        blake2b.update(&a.to_vec());
    });
    blake2b.update(old_ha.as_ref());
    blake2b.finalize(&mut buf);
    Hash::new(&buf)
}

