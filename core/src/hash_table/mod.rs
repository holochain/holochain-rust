pub mod entry;
pub mod header;
pub mod memory;
pub mod pair;
pub mod pair_meta;
pub mod status;

use agent::keys::Keys;
use error::HolochainError;
use hash_table::{pair::Pair, pair_meta::PairMeta};

pub trait HashTable {
    // internal state management
    fn setup(&mut self) -> Result<(), HolochainError>;
    fn teardown(&mut self) -> Result<(), HolochainError>;

    // crud
    /// add a Pair to the HashTable, analogous to chain.push() but ordering is not enforced
    fn commit(&mut self, pair: &Pair) -> Result<(), HolochainError>;
    /// lookup a Pair from the HashTable by Pair/Header key
    fn get(&self, key: &str) -> Result<Option<Pair>, HolochainError>;
    /// add a new Pair to the HashTable as per commit and status link an old Pair as MODIFIED
    fn modify(
        &mut self,
        keys: &Keys,
        old_pair: &Pair,
        new_pair: &Pair,
    ) -> Result<(), HolochainError>;
    /// set the status of a Pair to DELETED
    fn retract(&mut self, keys: &Keys, pair: &Pair) -> Result<(), HolochainError>;

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
