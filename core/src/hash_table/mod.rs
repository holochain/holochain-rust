pub mod actor;
pub mod entry;
pub mod file;
pub mod links_entry;
pub mod memory;
pub mod entry_meta;
pub mod status;
pub mod sys_entry;
#[cfg(test)]
pub mod test_util;

use agent::keys::Keys;
use error::HolochainError;
use hash::HashString;
use hash_table::{
    entry::Entry,
    entry_meta::EntryMeta,
    status::{CrudStatus, LINK_NAME, STATUS_NAME},
};
use key::Key;

/// Trait of the data structure storing the source chain
/// source chain is stored as a hash table of Headers and Entries.
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

    // CRUD
    /// Add an Entry to the HashTable, analogous to chain.push() but ordering is not enforced.
    fn put_entry(&mut self, entry: &Entry) -> Result<(), HolochainError>;
    /// Lookup an Entry from the HashTable by key.
    fn entry(&self, key: &HashString) -> Result<Option<Entry>, HolochainError>;

    /// Modify an existing Entry (by adding a new one and flagging the old one as MODIFIED)
    fn modify_entry(
        &mut self,
        keys: &Keys,
        old: &Entry,
        new: &Entry,
    ) -> Result<(), HolochainError> {
        // 1. Add a new Entry to the HashTable as per commit.
        self.put_entry(new)?;

        // 2. Set the crud-status EntryMeta of the old Entry to MODIFIED
        // @TODO what if meta fails when commit succeeds?
        // @see https://github.com/holochain/holochain-rust/issues/142
        self.assert_meta(&EntryMeta::new(
            &keys.node_id(),
            &old.key(),
            STATUS_NAME,
            &CrudStatus::MODIFIED.bits().to_string(),
        ))?;

        // 3. Update the crud-link EntryMeta of the new Entry to add the old Entry
        // @TODO what if meta fails when commit succeeds?
        // @see https://github.com/holochain/holochain-rust/issues/142
        self.assert_meta(&EntryMeta::new(
            &keys.node_id(),
            &old.key(),
            LINK_NAME,
            &new.key().to_str(),
        ))
    }

    /// Remove an Entry from the HashTable by flagging it DELETED
    fn retract_entry(&mut self, keys: &Keys, entry: &Entry) -> Result<(), HolochainError> {
        // Set the crud-status EntryMeta to DELETED
        self.assert_meta(&EntryMeta::new(
            &keys.node_id(),
            &entry.key(),
            STATUS_NAME,
            &CrudStatus::DELETED.bits().to_string(),
        ))
    }

    // Meta
    /// Assert a given Meta in the HashTable.
    fn assert_meta(&mut self, meta: &EntryMeta) -> Result<(), HolochainError>;
    /// Lookup a Meta from the HashTable by key.
    fn get_meta(&mut self, key: &HashString) -> Result<Option<EntryMeta>, HolochainError>;
    /// Lookup all Meta for a given Entry.
    fn metas_from_entry(&mut self, entry: &Entry) -> Result<Vec<EntryMeta>, HolochainError>;
    /// Lookup a Meta from a request.
    fn meta_from_request(
        &mut self,
        entry_hash: HashString,
        attribute_name: &str,
    ) -> Result<Option<EntryMeta>, HolochainError> {
        let key = EntryMeta::make_hash(&entry_hash, attribute_name);
        self.get_meta(&key)
    }

    // query
    // @TODO how should we handle queries?
    // @see https://github.com/holochain/holochain-rust/issues/141
    // fn query (&self, query: &Query) -> Result<std::collections::HashSet, HolochainError>;
}
