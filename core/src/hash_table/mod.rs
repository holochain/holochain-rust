pub mod actor;
pub mod entry;
pub mod file;
pub mod memory;
pub mod pair;
pub mod pair_meta;
pub mod status;
pub mod sys_entry;
#[cfg(test)]
pub mod test_util;

use agent::keys::Keys;
use error::HolochainError;
use hash_table::{
    pair::Pair,
    pair_meta::PairMeta,
    status::{CRUDStatus, LINK_NAME, STATUS_NAME},
};
use key::Key;

pub type HashString = String;

/// Trait of the data structure storing the source chain
/// source chain is stored as a hash table of Pairs.
/// Pair is a pair holding an Entry and its Header
pub trait HashTable: Send + Sync + Clone + 'static {
    // internal state management
    // @TODO does this make sense at the trait level?
    // @see https://github.com/holochain/holochain-rust/issues/262
    fn setup(&mut self) -> Result<(), HolochainError> {
        Ok(())
    }
    fn teardown(&mut self) -> Result<(), HolochainError> {
        Ok(())
    }

    // crud
    /// add a Pair to the HashTable, analogous to chain.push() but ordering is not enforced
    fn commit_pair(&mut self, pair: &Pair) -> Result<(), HolochainError>;

    /// lookup a Pair from the HashTable by Pair/Header key
    fn pair(&self, key: &str) -> Result<Option<Pair>, HolochainError>;

    /// add a new Pair to the HashTable as per commit and status link an old Pair as MODIFIED
    fn modify_pair(
        &mut self,
        keys: &Keys,
        old_pair: &Pair,
        new_pair: &Pair,
    ) -> Result<(), HolochainError> {
        self.commit_pair(new_pair)?;

        // @TODO what if meta fails when commit succeeds?
        // @see https://github.com/holochain/holochain-rust/issues/142
        self.assert_pair_meta(&PairMeta::new(
            keys,
            &old_pair,
            STATUS_NAME,
            &CRUDStatus::MODIFIED.bits().to_string(),
        ))?;

        // @TODO what if meta fails when commit succeeds?
        // @see https://github.com/holochain/holochain-rust/issues/142
        self.assert_pair_meta(&PairMeta::new(keys, &old_pair, LINK_NAME, &new_pair.key()))
    }

    /// set the status of a Pair to DELETED
    fn retract_pair(&mut self, keys: &Keys, pair: &Pair) -> Result<(), HolochainError> {
        self.assert_pair_meta(&PairMeta::new(
            keys,
            &pair,
            STATUS_NAME,
            &CRUDStatus::DELETED.bits().to_string(),
        ))
    }

    // meta
    /// assert a given PairMeta in the HashTable
    fn assert_pair_meta(&mut self, meta: &PairMeta) -> Result<(), HolochainError>;

    /// lookup a PairMeta from the HashTable by PairMeta key
    fn pair_meta(&mut self, key: &str) -> Result<Option<PairMeta>, HolochainError>;
    /// lookup all PairMeta for a given Pair
    fn metas_for_pair(&mut self, pair: &Pair) -> Result<Vec<PairMeta>, HolochainError>;

    // query
    // @TODO how should we handle queries?
    // @see https://github.com/holochain/holochain-rust/issues/141
    // fn query (&self, query: &Query) -> Result<std::collections::HashSet, HolochainError>;
}
