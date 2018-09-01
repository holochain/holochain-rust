pub mod actor;
pub mod deletion_entry;
pub mod entry;
pub mod links_entry;
pub mod memory;
pub mod meta;
pub mod status;
pub mod sys_entry;

use agent::keys::Keys;
use error::HolochainError;
use hash_table::{
    entry::Entry,
    links_entry::{Link, LinkListEntry},
    meta::Meta,
};
use nucleus::ribosome::api::get_links::GetLinksArgs;

pub type HashString = String;

/// Trait of the data structure storing the source chain
/// source chain is stored as a hash table of Pairs.
/// Pair is a pair holding an Entry and its Header
pub trait HashTable: Send + Sync + Clone + 'static {
    // internal state management
    // @TODO does this make sense at the trait level?
    // @see https://github.com/holochain/holochain-rust/issues/262
    fn setup(&mut self) -> Result<(), HolochainError>;
    fn teardown(&mut self) -> Result<(), HolochainError>;

    // CRUD
    /// Add an Entry to the HashTable, analogous to chain.push() but ordering is not enforced.
    fn put(&mut self, entry: &Entry) -> Result<(), HolochainError>;
    /// Lookup an Entry in the HashTable
    fn entry(&self, key: &str) -> Result<Option<Entry>, HolochainError>;
    /// Add a new Entry to the HashTable as put() and set status metadata on old Entry to MODIFIED.
    fn modify(
        &mut self,
        keys: &Keys,
        old_entry: &Entry,
        new_entry: &Entry,
    ) -> Result<(), HolochainError>;
    /// 'Remove' an Entry by setting the status metadata of an Entry to DELETED
    fn retract(&mut self, keys: &Keys, entry: &Entry) -> Result<(), HolochainError>;

    // Link
    /// Add link metadata to an Entry
    fn add_link(&mut self, link: &Link) -> Result<(), HolochainError>;
    /// Remove link metadata to an Entry
    fn remove_link(&mut self, link: &Link) -> Result<(), HolochainError>;
    /// Get all link metadata of an Entry
    fn links(
        &mut self,
        links_request: &GetLinksArgs,
    ) -> Result<Option<LinkListEntry>, HolochainError>;

    // Meta
    /// Assert a given Meta in the HashTable.
    fn assert_meta(&mut self, meta: &Meta) -> Result<(), HolochainError>;
    /// Lookup a Meta from the HashTable by key.
    fn meta(&mut self, key: &str) -> Result<Option<Meta>, HolochainError>;
    /// Lookup all Meta for a given Entry.
    fn meta_from_entry(&mut self, entry: &Entry) -> Result<Vec<Meta>, HolochainError>;
    /// Lookup a Meta from a request.
    fn meta_from_request(
        &mut self,
        entry_hash: HashString,
        attribute_name: &str,
    ) -> Result<Option<Meta>, HolochainError>;

    // query
    // @TODO how should we handle queries?
    // @see https://github.com/holochain/holochain-rust/issues/141
    // fn query (&self, query: &Query) -> Result<std::collections::HashSet, HolochainError>;
}
