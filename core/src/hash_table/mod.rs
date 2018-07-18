pub mod status;
pub mod entry;
pub mod header;
pub mod pair;
pub mod pair_meta;
pub mod memory;

use agent::keys::Keys;
use error::HolochainError;
use hash_table::pair::Pair;
use hash_table::pair_meta::PairMeta;

// https://stackoverflow.com/questions/30353462/how-to-clone-a-struct-storing-a-boxed-trait-object
pub trait HashTableClone {
    fn clone_box(&self) -> Box<HashTable>;
}

impl<HT> HashTableClone for HT
    where
        HT: 'static + HashTable + Clone,
        {
            fn clone_box(&self) -> Box<HashTable> {
                Box::new(self.clone())
            }
}

impl Clone for Box<HashTable> {
    fn clone(&self) -> Box<HashTable> {
        self.clone_box()
    }
}

pub trait HashTable: HashTableClone + Send + Sync {

    // state changes
    fn open (&mut self) -> Result<(), HolochainError>;
    fn close (&mut self) -> Result<(), HolochainError>;

    // crud
    /// add a Pair to the HashTable, analogous to chain.push() but ordering is not enforced
    fn commit (&mut self, pair: &Pair) -> Result<(), HolochainError>;
    /// lookup a Pair from the HashTable by Pair/Header key
    fn get (&self, key: &str) -> Result<Option<Pair>, HolochainError>;
    /// add a new Pair to the HashTable as per commit and status link an old Pair as MODIFIED
    fn modify (&mut self, keys: &Keys, old_pair: &Pair, new_pair: &Pair) -> Result<(), HolochainError>;
    /// set the status of a Pair to DELETED
    fn retract (&mut self, keys: &Keys, pair: &Pair) -> Result<(), HolochainError>;

    // meta
    /// assert a given PairMeta in the HashTable
    fn assert_meta(&mut self, meta: &PairMeta) -> Result<(), HolochainError>;
    /// lookup a PairMeta from the HashTable by key
    fn get_meta(&mut self, key: &str) -> Result<Option<PairMeta>, HolochainError>;
    /// lookup all PairMeta for a given Pair
    fn get_pair_meta(&mut self, pair: &Pair) -> Result<Vec<PairMeta>, HolochainError>;

    // query
    // @TODO how should we handle queries?
    // @see https://github.com/holochain/holochain-rust/issues/141
    // fn query (&self, query: &Query) -> Result<std::collections::HashSet, HolochainError>;

}
