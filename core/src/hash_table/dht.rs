use error::HolochainError;

use agent::keys::Keys;
use hash::HashString;
use hash_table::{entry::Entry, links_entry::*, meta::EntryMeta, sys_entry::ToEntry, HashTable};
use key::Key;
use nucleus::ribosome::api::get_links::GetLinksArgs;
use serde_json;
use std::collections::HashMap;

/// Struct implementing the HashTable Trait by storing the HashTable in memory
#[derive(Serialize, Debug, Clone, PartialEq, Default)]
pub struct Dht {
    entries: HashMap<HashString, Entry>,
    metas: HashMap<HashString, EntryMeta>,
}

impl HashTable for Dht {
    fn put_entry(&mut self, entry: &Entry) -> Result<(), HolochainError> {
        self.entries.insert(entry.key(), entry.clone());
        Ok(())
    }

    fn entry(&self, key: &HashString) -> Result<Option<Entry>, HolochainError> {
        Ok(self.entries.get(key).cloned())
    }

    fn assert_meta(&mut self, meta: &EntryMeta) -> Result<(), HolochainError> {
        self.metas.insert(meta.key(), meta.clone());
        Ok(())
    }

    fn get_meta(&mut self, key: &HashString) -> Result<Option<EntryMeta>, HolochainError> {
        Ok(self.metas.get(key).cloned())
    }

    /// Return all the Metas for an entry
    fn metas_from_entry(&mut self, entry: &Entry) -> Result<Vec<EntryMeta>, HolochainError> {
        let mut vec_meta = self
            .metas
            .values()
            .filter(|&m| m.entry_hash() == &entry.key())
            .cloned()
            .collect::<Vec<EntryMeta>>();
        // @TODO should this be sorted at all at this point?
        // @see https://github.com/holochain/holochain-rust/issues/144
        vec_meta.sort();
        Ok(vec_meta)
    }
}

impl Dht {
    /// Add link metadata to an Entry
    fn add_link(&mut self, link: &Link) -> Result<(), HolochainError> {
        // Retrieve entry from HashTable
        let base_entry = self.entry(&link.base())?;
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
                self.put_entry(&new_entry)?;

                // TODO #281 - should not have to create Keys
                let key_fixme = ::agent::keys::Key::new();
                let keys_fixme = Keys::new(&key_fixme, &key_fixme, "FIXME");

                // Create Meta
                new_meta = EntryMeta::new(
                    &keys_fixme.node_id(),
                    &base_entry.key(),
                    &link.to_attribute_name(),
                    &new_entry.key().to_str(),
                );
            }
            // Update existing LinkListEntry and Meta
            Some(meta) => {
                // Get LinkListEntry in HashTable
                let entry = self
                    .entry(&HashString::from(meta.value()))?
                    .expect("should have entry if meta points to it");
                let mut lle: LinkListEntry = serde_json::from_str(&entry.content())
                    .expect("entry is not a valid LinkListEntry");
                // Add Link
                lle.links.push(link.clone());
                // Make new Entry and commit it since it has changed
                let entry = lle.to_entry();
                // TODO maybe remove previous LinkListEntry ?
                self.put_entry(&entry)?;

                // Updated Meta to Assert
                assert!(meta.attribute() == link.to_attribute_name());
                new_meta = EntryMeta::new(
                    &meta.source(),
                    &base_entry.key(),
                    &meta.attribute(),
                    &entry.key().to_str(),
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
            .entry(&HashString::from(meta.value()))?
            .expect("should have entry listed in meta");
        Ok(Some(LinkListEntry::from_entry(&entry)))
    }
}

#[cfg(test)]
pub mod tests {
    use hash_table::{
        entry::Entry,
        links_entry::{Link, LinkListEntry},
        memory::MemTable,
        test_util::standard_suite,
        HashTable,
    };
    use key::Key;
    use nucleus::ribosome::api::get_links::GetLinksArgs;

    #[test]
    fn can_link_entries() {
        let mut table = MemTable::new();

        let e1 = Entry::new("app1", "abcdef");
        let e2 = Entry::new("app1", "qwerty");

        let t1 = "child".to_string();
        let t2 = "parent".to_string();

        let req1 = &GetLinksArgs {
            entry_hash: e1.key(),
            tag: t1.clone(),
        };
        let req2 = &GetLinksArgs {
            entry_hash: e1.key(),
            tag: t2.clone(),
        };

        let link = Link::new(&e1.key(), &e2.key(), &t1);

        table.put_entry(&e1).unwrap();
        table.put_entry(&e2).unwrap();

        assert_eq!(
            None,
            table.get_links(req1).expect("get_links() should not fail")
        );

        table.add_link(&link).unwrap();

        let lle = LinkListEntry::new(&[link]);

        assert_eq!(
            Some(lle),
            table.get_links(req1).expect("get_links() should not fail")
        );
        assert_eq!(
            None,
            table.get_links(req2).expect("get_links() should not fail")
        );
    }

    #[test]
    fn can_double_link_entries() {
        let mut table = MemTable::new();

        let e1 = Entry::new("app1", "abcdef");
        let e2 = Entry::new("app1", "qwerty");
        let e3 = Entry::new("app1", "fdfdsfds");

        let t1 = "child".to_string();

        let l1 = Link::new(&e1.key(), &e2.key(), &t1);
        let l2 = Link::new(&e1.key(), &e3.key(), &t1);

        let req1 = &GetLinksArgs {
            entry_hash: e1.key(),
            tag: t1.clone(),
        };

        table.put_entry(&e1).unwrap();
        table.put_entry(&e2).unwrap();
        table.put_entry(&e3).unwrap();

        table.add_link(&l1).unwrap();
        table.add_link(&l2).unwrap();

        let lle = LinkListEntry::new(&[l1, l2]);

        assert_eq!(
            Some(lle),
            table.get_links(req1).expect("get_links() should not fail")
        );
    }

    #[test]
    fn can_link_entries_adv() {
        let mut table = MemTable::new();

        let mom = Entry::new("app1", "abcdef");
        let son = Entry::new("app1", "qwerty");
        let daughter = Entry::new("app1", "fdfdsfds");

        let t1 = "child".to_string();
        let t2 = "parent".to_string();

        let mom_children = &GetLinksArgs {
            entry_hash: mom.key(),
            tag: t1.clone(),
        };
        let mom_parent = &GetLinksArgs {
            entry_hash: mom.key(),
            tag: t2.clone(),
        };
        let son_parent = &GetLinksArgs {
            entry_hash: son.key(),
            tag: t2.clone(),
        };
        let daughter_parent = &GetLinksArgs {
            entry_hash: daughter.key(),
            tag: t2.clone(),
        };
        let daughter_children = &GetLinksArgs {
            entry_hash: daughter.key(),
            tag: t1.clone(),
        };

        table.put_entry(&mom).unwrap();
        table.put_entry(&son).unwrap();
        table.put_entry(&daughter).unwrap();

        let mom_son = Link::new(&mom.key(), &son.key(), &t1);
        let son_mom = Link::new(&son.key(), &mom.key(), &t2);
        let mom_daughter = Link::new(&mom.key(), &daughter.key(), &t1);
        let daughter_mom = Link::new(&daughter.key(), &mom.key(), &t2);

        table.add_link(&mom_son).unwrap();
        table.add_link(&son_mom).unwrap();
        table.add_link(&mom_daughter).unwrap();
        table.add_link(&daughter_mom).unwrap();

        let res_children = LinkListEntry::new(&[mom_son, mom_daughter]);
        let res_son_parent = LinkListEntry::new(&[son_mom]);
        let res_daughter_parent = LinkListEntry::new(&[daughter_mom]);

        assert_eq!(
            None,
            table
                .get_links(daughter_children)
                .expect("get_links() should not fail")
        );
        assert_eq!(
            None,
            table
                .get_links(mom_parent)
                .expect("get_links() should not fail")
        );
        assert_eq!(
            Some(res_children),
            table
                .get_links(mom_children)
                .expect("get_links() should not fail")
        );
        assert_eq!(
            Some(res_son_parent),
            table
                .get_links(son_parent)
                .expect("get_links() should not fail")
        );
        assert_eq!(
            Some(res_daughter_parent),
            table
                .get_links(daughter_parent)
                .expect("get_links() should not fail")
        );
    }
}
