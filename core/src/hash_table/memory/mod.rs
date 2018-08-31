use std::collections::HashMap;

use error::HolochainError;
use serde_json;
use agent::keys::Key;
use agent::keys::Keys;
use hash_table::{
    pair_meta::Meta,
    // status::{CRUDStatus, LINK_NAME, STATUS_NAME},
    HashTable,
    links_entry::Link,
    HashString,
    links_entry::LinkListEntry,
    sys_entry::ToEntry,
    entry::Entry,
};
use nucleus::ribosome::api::get_links::GetLinksArgs;

/// Struct implementing the HashTable Trait by storing the HashTable in memory
#[derive(Serialize, Debug, Clone, PartialEq, Default)]
pub struct MemTable {
    entries: HashMap<String, Entry>,
    metas: HashMap<String, Meta>,
}

impl MemTable {
    pub fn new() -> MemTable {
        MemTable {
            entries: HashMap::new(),
            metas: HashMap::new(),
        }
    }
}

impl HashTable for MemTable {
    fn setup(&mut self) -> Result<(), HolochainError> {
        Ok(())
    }

    fn teardown(&mut self) -> Result<(), HolochainError> {
        Ok(())
    }

    fn put(&mut self, entry: &Entry) -> Result<(), HolochainError> {
        self.entries.insert(entry.key(), entry.clone());
        Ok(())
    }

    fn entry(&self, key: &str) -> Result<Option<Entry>, HolochainError> {
        Ok(self.entries.get(key).cloned())
    }

//    fn modify(
//        &mut self,
//        keys: &Keys,
//        old_pair: &Pair,
//        new_pair: &Pair,
//    ) -> Result<(), HolochainError> {
//        self.commit(new_pair)?;
//
//        // @TODO what if meta fails when commit succeeds?
//        // @see https://github.com/holochain/holochain-rust/issues/142
//        self.assert_meta(&Meta::new(
//            keys,
//            &old_pair.key(),
//            STATUS_NAME,
//            &CRUDStatus::MODIFIED.bits().to_string(),
//        ))?;
//
//        // @TODO what if meta fails when commit succeeds?
//        // @see https://github.com/holochain/holochain-rust/issues/142
//        self.assert_meta(&Meta::new(keys, &old_pair.key(), LINK_NAME, &new_pair.key()))
//    }

//    fn retract(&mut self, keys: &Keys, pair: &Pair) -> Result<(), HolochainError> {
//        self.assert_meta(&Meta::new(
//            keys,
//            &pair.key(),
//            STATUS_NAME,
//            &CRUDStatus::DELETED.bits().to_string(),
//        ))
//    }

    // Add Link Meta to an entry
    fn add_link(&mut self, link: &Link) -> Result<(), HolochainError> {
        // Retrieve entry from HashTable
        let base_entry = self.entry(&link.base())?;
        if base_entry.is_none() {
            return Err(HolochainError::ErrorGeneric("Pair from base not found".to_string()));
        }
        let base_entry = base_entry.unwrap();

        // pre-condition: linking must only work on AppEntries
        if base_entry.is_sys() {
            return Err(HolochainError::InvalidOperationOnSysEntry);
        }

        // Retrieve LinkListEntry
        let maybe_meta = self.get_meta_for(base_entry.key(), &link.to_attribute_name())?;
        // Update or Create LinkListEntry
        let mut new_meta: Meta;
        match maybe_meta {
            // None found so create one
            None => {
                // Create new LinkListEntry & Pair
                let lle = LinkListEntry::new(&[link.clone()]);
                let new_entry = lle.to_entry();
                // Add it to HashTable
                self.put(&new_entry)?;

                // TODO - should not have to create Keys
                let key_fixme = Key::new();
                let keys_fixme = Keys::new(&key_fixme, &key_fixme, "FIXME");

                // Create PairMeta
                new_meta = Meta::new(
                    &keys_fixme.node_id(),
                    &base_entry.key(),
                    &link.to_attribute_name(),
                    &new_entry.key());
            }
            // Update existing LinkListEntry and Meta
            Some(meta) => {
                // Get LinkListEntry in HashTable
                let entry = self.entry(&meta.value())?
                    .expect("should have entry if meta points to it");
                // assert!(entry.entry_type())
                let mut lle: LinkListEntry = serde_json::from_str(&entry.content())
                    .expect("entry is not a valid LinkListEntry");
                // Add Link
                lle.links.push(link.clone());
                // Make new Entry and commit it since it has changed
                let entry = lle.to_entry();
                // TODO maybe remove previous LinkListEntry ?
                self.put(&entry)?;

                // Updated PairMeta to Assert
                assert!(meta.attribute() == link.to_attribute_name());
                new_meta = Meta::new(
                    &meta.source(),
                    &base_entry.key(),
                    &meta.attribute(),
                    &entry.key());
            }
        }

        // Insert new/changed PairMeta
        self.assert_meta(&new_meta).expect("meta should be valid");

        // Done
        Ok(())
    }

    // Remove link from a LinkListEntry entry from Meta
    fn remove_link(&mut self, _link: &Link) -> Result<(), HolochainError> {
        // TODO
        Err(HolochainError::NotImplemented)
    }

    // Get all links from an AppEntry by using metadata
    fn links(&mut self, request: &GetLinksArgs) -> Result<Option<LinkListEntry>, HolochainError> {
        // Look for entry's metadata
        let vec_meta = self.get_meta_for(request.clone().entry_hash, &request.to_attribute_name())?;
        if vec_meta.is_none() {
            return Ok(None);
        }
        let meta = vec_meta.unwrap();

        // Get LinkListEntry in HashTable
        let entry = self.entry(&meta.value())?.expect("should have entry listed in meta");
        Ok(Some(LinkListEntry::new_from_entry(&entry)))
    }


    fn assert_meta(&mut self, meta: &Meta) -> Result<(), HolochainError> {
        self.metas.insert(meta.hash(), meta.clone());
        Ok(())
    }

    /// Return a Meta from a Meta.key
    fn get_meta(&mut self, key: &str) -> Result<Option<Meta>, HolochainError> {
        Ok(self.metas.get(key).cloned())
    }

    /// Return all the Metas for an entry
    fn get_entry_meta(&mut self, entry: &Entry) -> Result<Vec<Meta>, HolochainError> {
        let mut vec_meta = self
            .metas
            .values()
            .filter(|&m| m.entry_hash() == entry.key())
            .cloned()
            .collect::<Vec<Meta>>();
        // @TODO should this be sorted at all at this point?
        // @see https://github.com/holochain/holochain-rust/issues/144
        vec_meta.sort();
        Ok(vec_meta)
    }

    /// Return a Meta from an entry_hash and attribute_name
    fn get_meta_for(&mut self, entry_hash: HashString, attribute_name: &str) -> Result<Option<Meta>, HolochainError>
    {
        let key = Meta::make_hash(&entry_hash, attribute_name);
        self.get_meta(&key)
    }
}

#[cfg(test)]
pub mod tests {
    // use agent::keys::tests::test_keys;
    use hash_table::{
        links_entry::Link, links_entry::LinkListEntry,
        sys_entry::ToEntry,
        memory::MemTable,
        pair::tests::{test_pair,
                       test_pair_a, test_pair_b,
        },
        pair_meta::{
            tests::{test_pair_meta,
                     test_pair_meta_a, test_pair_meta_b,
            },
             Meta,
        },
        // status::{CRUDStatus, LINK_NAME, STATUS_NAME},
        HashTable,
        actor::HashTableActor,
        entry::Entry,
    };
//use ::chain::{
//     Chain,
//     SourceChain,
//};
    use nucleus::ribosome::api::get_links::GetLinksArgs;

    pub fn test_table() -> MemTable {
        MemTable::new()
    }

    #[test]
    /// smoke test
    fn new() {
        test_table();
    }

    #[test]
    /// tests for ht.setup()
    fn setup() {
        let mut ht = test_table();
        assert_eq!(Ok(()), ht.setup());
    }

    #[test]
    /// tests for ht.teardown()
    fn teardown() {
        let mut ht = test_table();
        assert_eq!(Ok(()), ht.teardown());
    }

    #[test]
    /// Pairs can round trip through table.commit() and table.get()
    fn pair_round_trip() {
        let mut table = test_table();
        let pair = test_pair();
        table
            .put(&pair.header().to_entry())
            .expect("should be able to commit valid pair");
        assert_eq!(table.entry(&pair.key()).unwrap().unwrap(), pair.header().to_entry());
    }

//    #[test]
//    /// Pairs can be modified through table.modify()
//    fn modify() {
//        let mut ht = test_table();
//        let p1 = test_pair_a();
//        let p2 = test_pair_b();
//
//        ht.commit(&p1).expect("should be able to commit valid pair");
//        ht.modify(&test_keys(), &p1, &p2)
//            .expect("should be able to edit with valid pair");
//
//        assert_eq!(
//            vec![
//                Meta::new(&test_keys(), &p1, LINK_NAME, &p2.key()),
//                Meta::new(
//                    &test_keys(),
//                    &p1,
//                    STATUS_NAME,
//                    &CRUDStatus::MODIFIED.bits().to_string(),
//                ),
//            ],
//            ht.get_pair_meta(&p1)
//                .expect("getting the metadata on a pair shouldn't fail")
//        );
//
//        let empty_vec: Vec<Meta> = Vec::new();
//        assert_eq!(
//            empty_vec,
//            ht.get_pair_meta(&p2)
//                .expect("getting the metadata on a pair shouldn't fail")
//        );
//    }

//    #[test]
//    /// Pairs can be retracted through table.retract()
//    fn retract() {
//        let mut ht = test_table();
//        let p = test_pair();
//        let empty_vec: Vec<Meta> = Vec::new();
//
//        ht.commit(&p).expect("should be able to commit valid pair");
//        assert_eq!(
//            empty_vec,
//            ht.get_pair_meta(&p)
//                .expect("getting the metadata on a pair shouldn't fail")
//        );
//
//        ht.retract(&test_keys(), &p)
//            .expect("should be able to retract");
//        assert_eq!(
//            vec![Meta::new(
//                &test_keys(),
//                &p,
//                STATUS_NAME,
//                &CRUDStatus::DELETED.bits().to_string(),
//            )],
//            ht.get_pair_meta(&p)
//                .expect("getting the metadata on a pair shouldn't fail"),
//        );
//    }

    #[test]
    /// PairMeta can round trip through table.assert_meta() and table.get_meta()
    fn meta_round_trip() {
        let mut table = test_table();
        let pair_meta = test_pair_meta();

        assert_eq!(
            None,
            table
                .get_meta(&pair_meta.hash())
                .expect("getting the metadata on a pair shouldn't fail")
        );

        table
            .assert_meta(&pair_meta)
            .expect("asserting metadata shouldn't fail");
        assert_eq!(
            Some(&pair_meta),
            table
                .get_meta(&pair_meta.hash())
                .expect("getting the metadata on a pair shouldn't fail")
                .as_ref()
        );
    }

    #[test]
    /// all Meta for an Entry can be retrieved with get_entry_meta
    fn can_get_entry_meta() {
        let mut table = test_table();
        let pair = test_pair();
        let pair_meta_a = test_pair_meta_a();
        let pair_meta_b = test_pair_meta_b();
        let empty_vec: Vec<Meta> = Vec::new();

        let pair_entry = pair.header().to_entry();

        assert_eq!(
            empty_vec,
            table
                .get_entry_meta(&pair_entry)
                .expect("getting the metadata on a pair shouldn't fail")
        );

        table
            .assert_meta(&pair_meta_a)
            .expect("asserting metadata shouldn't fail");
        assert_eq!(
            vec![pair_meta_a.clone()],
            table
                .get_entry_meta(&pair_entry)
                .expect("getting the metadata on a pair shouldn't fail")
        );

        table
            .assert_meta(&pair_meta_b.clone())
            .expect("asserting metadata shouldn't fail");
        assert_eq!(
            vec![pair_meta_b.clone(), pair_meta_a.clone()],
            table
                .get_entry_meta(&pair_entry)
                .expect("getting the metadata on a pair shouldn't fail")
        );


        // test meta_for
        assert_eq!(
            Some(pair_meta_a.clone()),
            table
                .get_meta_for(pair_entry.key(), &pair_meta_a.attribute())
                .expect("getting the metadata on a pair shouldn't fail")
        );
        assert_eq!(
            Some(pair_meta_b.clone()),
            table
                .get_meta_for(pair_entry.key(), &pair_meta_b.attribute())
                .expect("getting the metadata on a pair shouldn't fail")
        );
    }


    #[test]
    fn can_link_entries() {
        let mut table = MemTable::new();

        let e1 = Entry::new("app1", "abcdef");
        let e2 = Entry::new("app1", "qwerty");

        let t1 = "child".to_string();
        let t2 = "parent".to_string();

        let req1 = &GetLinksArgs{entry_hash:e1.key(), tag: t1.clone()};
        let req2 = &GetLinksArgs{entry_hash:e1.key(), tag: t2.clone()};

        let link = Link::new(&e1.key(), &e2.key(), &t1);

        table.put(&e1).unwrap();
        table.put(&e2).unwrap();

        assert_eq!(
            None,
            table.links(req1)
                 .expect("links() should not fail"));


        table.add_link(&link).unwrap();

        let lle = LinkListEntry::new(&[link]);

        assert_eq!(
            Some(lle),
            table.links(req1)
                .expect("links() should not fail"));
        assert_eq!(
            None,
            table.links(req2)
                 .expect("links() should not fail"));
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

        let req1 = &GetLinksArgs{entry_hash:e1.key(), tag: t1.clone()};

        table.put(&e1).unwrap();
        table.put(&e2).unwrap();
        table.put(&e3).unwrap();


        table.add_link(&l1).unwrap();
        table.add_link(&l2).unwrap();

        let lle = LinkListEntry::new(&[l1, l2]);

        assert_eq!(
            Some(lle),
            table.links(req1)
                 .expect("links() should not fail"));

//        can_double_same_link_entries() {
//            // TODO
//        }
    }


    #[test]
    fn can_link_entries_adv() {
        let mut table = MemTable::new();

        let mom = Entry::new("app1", "abcdef");
        let son = Entry::new("app1", "qwerty");
        let daughter = Entry::new("app1", "fdfdsfds");

        let t1 = "child".to_string();
        let t2 = "parent".to_string();

        let mom_children = &GetLinksArgs{entry_hash:mom.key(), tag: t1.clone()};
        let mom_parent = &GetLinksArgs{entry_hash:mom.key(), tag: t2.clone()};
        let son_parent = &GetLinksArgs{entry_hash:son.key(), tag: t2.clone()};
        let daughter_parent = &GetLinksArgs{entry_hash:daughter.key(), tag: t2.clone()};
        let daughter_children = &GetLinksArgs{entry_hash:daughter.key(), tag: t1.clone()};

        table.put(&mom).unwrap();
        table.put(&son).unwrap();
        table.put(&daughter).unwrap();

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
            table.links(daughter_children)
                 .expect("links() should not fail"));
        assert_eq!(
            None,
            table.links(mom_parent)
                 .expect("links() should not fail"));
        assert_eq!(
            Some(res_children),
            table.links(mom_children)
                 .expect("links() should not fail"));
        assert_eq!(
            Some(res_son_parent),
            table.links(son_parent)
                 .expect("links() should not fail"));
        assert_eq!(
            Some(res_daughter_parent),
            table.links(daughter_parent)
                 .expect("links() should not fail"));
    }
}
