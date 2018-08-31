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
        // println!("MemTable.put = {}", entry.key());
        // println!("\t\tMemTable.put = {:?}", entry);
        self.entries.insert(entry.key(), entry.clone());
        Ok(())
    }

    fn entry(&self, key: &str) -> Result<Option<Entry>, HolochainError> {
        //println!("MemTable.GetEntry = {}", key);
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

    // Add Link Meta to a Pair
    fn add_link(&mut self, link: &Link) -> Result<(), HolochainError> {
        // Retrieve Pair from HashTable
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
        let mut maybe_meta = self.get_meta_for(base_entry.key(), &link.to_attribute_name())?;
        // Update or Create LinkListEntry
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
                maybe_meta = Some(Meta::new(
                    &keys_fixme.node_id(),
                    &base_entry.key(),
                    &link.to_attribute_name(),
                    &new_entry.key()));
            }
            // Update existing LinkListEntry and Meta
            Some(meta) => {
                // Get LinkListEntry in HashTable
                let entry = self.entry(&meta.value())?
                    .expect("should have entry if meta points to it");
                let mut lle : LinkListEntry = serde_json::from_str(&entry.content())
                    .expect("entry is not a valid LinkListEntry");
                // Add Link
                lle.links.push(link.clone());
                // Make new Entry and commit it since it has changed
                let entry = lle.to_entry();
                // TODO maybe remove previous LinkListEntry ?
                self.put(&entry)?;

                // Push new PairMeta
                assert!(meta.attribute() == link.to_attribute_name());
                maybe_meta = Some(Meta::new(
                    &meta.source(),
                    &base_entry.key(),
                    &meta.attribute(),
                    &entry.key()));
            }
        }

        // Insert new/changed PairMeta
        self.assert_meta(&maybe_meta.unwrap()).expect("meta should be valid");

        // Done
        Ok(())
    }

    // Remove Link from a LinkMeta
    fn remove_link(&mut self, _link: &Link) -> Result<(), HolochainError> {
        // TODO
        Err(HolochainError::NotImplemented)
    }

    // Get all links from an AppEntry by using metadata
    fn links(&mut self, request: &GetLinksArgs) -> Result<Option<LinkListEntry>, HolochainError> {
        // TODO - Check that is not a system entry?
        // Look for entry's metadata
        let result = self.get_meta_for(request.clone().entry_hash, &request.to_attribute_name())?;
        if result.clone().is_none() {
            return Ok(None);
        }
        let meta = result.unwrap();

        // Get LinkListEntry in HashTable
        let entry = self.entry(&meta.value())?.expect("should have entry listed in meta");
        Ok(Some(LinkListEntry::new_from_entry(&entry)))
    }


    fn assert_meta(&mut self, meta: &Meta) -> Result<(), HolochainError> {
        self.metas.insert(meta.hash(), meta.clone());
        Ok(())
    }


    // Return a Meta from a Meta key
    fn get_meta(&mut self, key: &str) -> Result<Option<Meta>, HolochainError> {
        Ok(self.metas.get(key).cloned())
    }


//    fn get_pair_meta(&mut self, pair: &Pair) -> Result<Vec<Meta>, HolochainError> {
//        let mut vec_meta = self
//            .metas
//            .values()
//            .filter(|&m| m.entity_hash() == pair.key())
//            .cloned()
//            .collect::<Vec<Meta>>();
//        // @TODO should this be sorted at all at this point?
//        // @see https://github.com/holochain/holochain-rust/issues/144
//        vec_meta.sort();
//        Ok(vec_meta)
//    }


    // Return all the Meta for an entry
    fn get_entry_meta(&mut self, entry: &Entry) -> Result<Vec<Meta>, HolochainError> {
        let mut vec_meta = self
            .metas
            .values()
            .filter(|&m| m.entity_hash() == entry.key())
            .cloned()
            .collect::<Vec<Meta>>();
        // @TODO should this be sorted at all at this point?
        // @see https://github.com/holochain/holochain-rust/issues/144
        vec_meta.sort();
        Ok(vec_meta)
    }

    // ;)
    fn get_meta_for(&mut self, entry_hash: HashString, attribute_name: &str) -> Result<Option<Meta>, HolochainError>
    {
        let vec_meta = self
            .metas
            .values()
            .filter(|&m| m.entity_hash() == entry_hash && m.attribute() == attribute_name)
            .cloned()
            .collect::<Vec<Meta>>();
        assert!(vec_meta.len() <= 1);
        Ok(if vec_meta.len() == 0 { None } else { Some(vec_meta[0].clone()) })
    }

}

#[cfg(test)]
pub mod tests {

    // use agent::keys::tests::test_keys;
    use hash_table::{
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
    };

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


//    #[test]
//    fn can_add_link() {
//        let mut table = MemTable::new();
//        let pair = Pair::new_from_chain(Chain::new(HashTableActor::new_ref(table)), &test_entry())
//        let pair_entry = pair.header().to_entry();
//
//        let link = Link::new();
//        table.add_link();
//    }
}
