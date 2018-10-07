pub mod actor;
pub mod file;
pub mod memory;
pub mod status;
pub mod sys_entry;
#[cfg(test)]
pub mod test_util;

use holochain_core_types::{
    cas::content::{Address, AddressableContent},
    entry::Entry, entry_meta::EntryMeta,
    error::HolochainError,
    keys::Keys,
};
use hash_table::{
    status::{CrudStatus, LINK_NAME, STATUS_NAME},
};

/// Trait of the data structure storing the source chain
/// source chain is stored as a hash table of ChainHeaders and Entries.
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
    fn entry(&self, address: &Address) -> Result<Option<Entry>, HolochainError>;

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
            &old.address(),
            STATUS_NAME,
            &CrudStatus::MODIFIED.bits().to_string(),
        ))?;

        // 3. Update the crud-link EntryMeta of the new Entry to add the old Entry
        // @TODO what if meta fails when commit succeeds?
        // @see https://github.com/holochain/holochain-rust/issues/142
        self.assert_meta(&EntryMeta::new(
            &keys.node_id(),
            &old.address(),
            LINK_NAME,
            &new.address().to_string(),
        ))
    }

    /// Remove an Entry from the HashTable by flagging it DELETED
    fn retract_entry(&mut self, keys: &Keys, entry: &Entry) -> Result<(), HolochainError> {
        // Set the crud-status EntryMeta to DELETED
        self.assert_meta(&EntryMeta::new(
            &keys.node_id(),
            &entry.address(),
            STATUS_NAME,
            &CrudStatus::DELETED.bits().to_string(),
        ))
    }

    // Meta
    /// Assert a given Meta in the HashTable.
    fn assert_meta(&mut self, meta: &EntryMeta) -> Result<(), HolochainError>;
    /// Lookup a Meta from the HashTable by address.
    fn get_meta(&mut self, address: &Address) -> Result<Option<EntryMeta>, HolochainError>;
    /// Lookup all Meta for a given Entry.
    fn metas_from_entry(&mut self, entry: &Entry) -> Result<Vec<EntryMeta>, HolochainError>;
    /// Lookup a Meta from a request.
    fn meta_from_request(
        &mut self,
        entry_address: Address,
        attribute_name: &str,
    ) -> Result<Option<EntryMeta>, HolochainError> {
        let address = EntryMeta::make_address(&entry_address, attribute_name);
        self.get_meta(&address)
    }

    // query
    // @TODO how should we handle queries?
    // @see https://github.com/holochain/holochain-rust/issues/141
    // fn query (&self, query: &Query) -> Result<std::collections::HashSet, HolochainError>;
}
