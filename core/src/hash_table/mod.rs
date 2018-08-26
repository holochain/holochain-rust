pub mod entry;
pub mod header;
pub mod memory;
pub mod file;
pub mod pair;
pub mod pair_meta;
pub mod status;
#[cfg(test)]
pub mod test_util;

use agent::keys::Keys;
use error::HolochainError;
use hash_table::{pair::Pair, pair_meta::PairMeta};
use hash_table::status::CRUDStatus;
use hash_table::status::STATUS_NAME;
use hash_table::status::LINK_NAME;

pub trait HashTable {
    // internal state management
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
        self.assert_pair_meta(PairMeta::new(
            keys,
            &old_pair,
            STATUS_NAME,
            &CRUDStatus::MODIFIED.bits().to_string(),
        ))?;

        // @TODO what if meta fails when commit succeeds?
        // @see https://github.com/holochain/holochain-rust/issues/142
        self.assert_pair_meta(PairMeta::new(keys, &old_pair, LINK_NAME, &new_pair.key()))
    }

    /// set the status of a Pair to DELETED
    fn retract_pair(&mut self, keys: &Keys, pair: &Pair) -> Result<(), HolochainError> {
        self.assert_pair_meta(PairMeta::new(
            keys,
            &pair,
            STATUS_NAME,
            &CRUDStatus::DELETED.bits().to_string(),
        ))
    }

    // meta
    /// assert a given PairMeta in the HashTable
    fn assert_pair_meta(&mut self, meta: PairMeta) -> Result<(), HolochainError>;
    /// lookup a PairMeta from the HashTable by key
    fn pair_meta(&mut self, key: &str) -> Result<Option<PairMeta>, HolochainError>;
    /// lookup all PairMeta for a given Pair
    fn all_metas_for_pair(&mut self, pair: &Pair) -> Result<Vec<PairMeta>, HolochainError>;

    // query
    // @TODO how should we handle queries?
    // @see https://github.com/holochain/holochain-rust/issues/141
    // fn query (&self, query: &Query) -> Result<std::collections::HashSet, HolochainError>;
}
