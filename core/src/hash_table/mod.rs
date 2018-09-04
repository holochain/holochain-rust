pub mod actor;
pub mod entry;
pub mod links_entry;
pub mod file;
pub mod memory;
pub mod meta;
pub mod status;
pub mod sys_entry;
#[cfg(test)]
pub mod test_util;

use agent::keys::Keys;
use error::HolochainError;
use hash_table::{
    entry::Entry,
    links_entry::{Link, LinkListEntry},
    meta::EntryMeta,
    status::{CrudStatus, LINK_NAME, STATUS_NAME},
    sys_entry::ToEntry,
};
use key::Key;
use nucleus::ribosome::api::get_links::GetLinksArgs;
use serde_json;

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

    // CRUD
    /// Add an Entry to the HashTable, analogous to chain.push() but ordering is not enforced.
    fn put(&mut self, entry: &Entry) -> Result<(), HolochainError>;
    /// Lookup an Entry from the HashTable by key.
    fn get(&self, key: &str) -> Result<Option<Entry>, HolochainError>;

    /// Modify an existing Entry (by adding a new one and flagging the old one as MODIFIED)
    fn modify(&mut self, keys: &Keys, old: &Entry, new: &Entry) -> Result<(), HolochainError> {
        // 1. Add a new Entry to the HashTable as per commit.
        self.put(new)?;

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
            &new.key(),
        ))
    }

    /// Remove an Entry from the HashTable by flagging it DELETED
    fn retract(&mut self, keys: &Keys, entry: &Entry) -> Result<(), HolochainError> {
        // Set the crud-status EntryMeta to DELETED
        self.assert_meta(&EntryMeta::new(
            &keys.node_id(),
            &entry.key(),
            STATUS_NAME,
            &CrudStatus::DELETED.bits().to_string(),
        ))
    }

    /// Add link metadata to an Entry
    fn add_link(&mut self, link: &Link) -> Result<(), HolochainError> {
        // Retrieve entry from HashTable
        let base_entry = self.get(&link.base())?;
        if base_entry.is_none() {
            return Err(HolochainError::ErrorGeneric(
                "Entry from base not found".to_string(),
            ));
        }
        let base_entry = base_entry.unwrap();

        // pre-condition: linking must only work on AppEntries
        if base_entry.is_sys() {
            return Err(HolochainError::InvalidOperationOnSysEntry);
        }

        // Retrieve LinkListEntry
        let maybe_meta = self.meta_from_request(base_entry.key(), &link.to_attribute_name())?;
        // Update or Create LinkListEntry
        let new_meta: EntryMeta;
        match maybe_meta {
            // None found so create one
            None => {
                // Create new LinkListEntry & Entry
                let lle = LinkListEntry::new(&[link.clone()]);
                let new_entry = lle.to_entry();
                // Add it to HashTable
                self.put(&new_entry)?;

                // TODO #281 - should not have to create Keys
                let key_fixme = ::agent::keys::Key::new();
                let keys_fixme = Keys::new(&key_fixme, &key_fixme, "FIXME");

                // Create Meta
                new_meta = EntryMeta::new(
                    &keys_fixme.node_id(),
                    &base_entry.key(),
                    &link.to_attribute_name(),
                    &new_entry.key(),
                );
            }
            // Update existing LinkListEntry and Meta
            Some(meta) => {
                // Get LinkListEntry in HashTable
                let entry = self
                    .get(&meta.value())?
                    .expect("should have entry if meta points to it");
                let mut lle: LinkListEntry = serde_json::from_str(&entry.content())
                    .expect("entry is not a valid LinkListEntry");
                // Add Link
                lle.links.push(link.clone());
                // Make new Entry and commit it since it has changed
                let entry = lle.to_entry();
                // TODO maybe remove previous LinkListEntry ?
                self.put(&entry)?;

                // Updated Meta to Assert
                assert!(meta.attribute() == link.to_attribute_name());
                new_meta = EntryMeta::new(
                    &meta.source(),
                    &base_entry.key(),
                    &meta.attribute(),
                    &entry.key(),
                );
            }
        }

        // Insert new/changed Meta
        self.assert_meta(&new_meta).expect("meta should be valid");

        // Done
        Ok(())
    }

    // Remove link from a LinkListEntry entry from Meta
    fn remove_link(&mut self, _link: &Link) -> Result<(), HolochainError> {
        // TODO #278 - Removable links features
        Err(HolochainError::NotImplemented)
    }

    // Get all links from an AppEntry by using metadata
    fn get_links(
        &mut self,
        request: &GetLinksArgs,
    ) -> Result<Option<LinkListEntry>, HolochainError> {
        // Look for entry's metadata
        let vec_meta =
            self.meta_from_request(request.clone().entry_hash, &request.to_attribute_name())?;
        if vec_meta.is_none() {
            return Ok(None);
        }
        let meta = vec_meta.unwrap();

        // Get LinkListEntry in HashTable
        let entry = self
            .get(&meta.value())?
            .expect("should have entry listed in meta");
        Ok(Some(LinkListEntry::from_entry(&entry)))
    }

    // Meta
    /// Assert a given Meta in the HashTable.
    fn assert_meta(&mut self, meta: &EntryMeta) -> Result<(), HolochainError>;
    /// Lookup a Meta from the HashTable by key.
    fn get_meta(&mut self, key: &str) -> Result<Option<EntryMeta>, HolochainError>;
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
