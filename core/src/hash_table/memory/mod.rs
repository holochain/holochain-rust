use error::HolochainError;

use hash_table::{entry::Entry, meta::EntryMeta, HashTable};
use key::Key;
use std::collections::HashMap;
use hash::HashString;

/// Struct implementing the HashTable Trait by storing the HashTable in memory
#[derive(Serialize, Debug, Clone, PartialEq, Default)]
pub struct MemTable {
    entries: HashMap<String, Entry>,
    metas: HashMap<String, EntryMeta>,
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
            .filter(|&m| m.entry_hash() == entry.key())
            .cloned()
            .collect::<Vec<EntryMeta>>();
        // @TODO should this be sorted at all at this point?
        // @see https://github.com/holochain/holochain-rust/issues/144
        vec_meta.sort();
        Ok(vec_meta)
    }
}

#[cfg(test)]
pub mod tests {

    use agent::keys::tests::test_keys;
    use hash_table::{
        entry::{tests::test_entry, Entry},
        links_entry::{Link, LinkListEntry},
        memory::MemTable,
        meta::{
            tests::{test_meta_a, test_meta_b},
            EntryMeta,
        },
        status::{CrudStatus, LINK_NAME, STATUS_NAME},
        test_util::standard_suite,
        HashTable,
    };
    use key::Key;
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
    /// An Entry can round trip through table.put() and table.entry()
    fn entry_round_trip() {
        let mut table = test_table();
        let e1 = Entry::new("t1", "e1");
        table
            .put_entry(&e1)
            .expect("should be able to commit valid entry");
        assert_eq!(e1, table.entry(&e1.key()).unwrap().unwrap());
    }

    #[test]
    /// Entries can be modified through table.modify()
    fn modify() {
        let mut ht = test_table();
        let e1 = Entry::new("t1", "c1");
        let e2 = Entry::new("t2", "c2");

        ht.put_entry(&e1)
            .expect("should be able to commit valid entry");
        ht.modify_entry(&test_keys(), &e1, &e2)
            .expect("should be able to edit with valid entry");

        assert_eq!(
            vec![
                EntryMeta::new(&test_keys().node_id(), &e1.key(), LINK_NAME, &e2.key()),
                EntryMeta::new(
                    &test_keys().node_id(),
                    &e1.key(),
                    STATUS_NAME,
                    &CrudStatus::MODIFIED.bits().to_string(),
                ),
            ],
            ht.metas_from_entry(&e1)
                .expect("getting the metadata on a entry shouldn't fail")
        );

        let empty_vec: Vec<EntryMeta> = Vec::new();
        assert_eq!(
            empty_vec,
            ht.metas_from_entry(&e2)
                .expect("getting the metadata on a entry shouldn't fail")
        );
    }

    #[test]
    /// an Entry can be retracted through table.retract()
    fn retract() {
        let mut ht = test_table();
        let e1 = Entry::new("t1", "c1");
        let empty_vec: Vec<EntryMeta> = Vec::new();

        ht.put_entry(&e1)
            .expect("should be able to commit valid entry");
        assert_eq!(
            empty_vec,
            ht.metas_from_entry(&e1)
                .expect("getting the metadata on a entry shouldn't fail")
        );

        ht.retract_entry(&test_keys(), &e1)
            .expect("should be able to retract");
        assert_eq!(
            vec![EntryMeta::new(
                &test_keys().node_id(),
                &e1.key(),
                STATUS_NAME,
                &CrudStatus::DELETED.bits().to_string(),
            )],
            ht.metas_from_entry(&e1)
                .expect("getting the metadata on a entry shouldn't fail"),
        );
    }

    #[test]
    /// Meta can round trip through table.assert_meta() and table.meta()
    fn meta_round_trip() {
        let mut table = test_table();
        let meta = EntryMeta::new("42", &"0x42".to_string(), "name", "toto");

        assert_eq!(
            None,
            table
                .get_meta(&meta.key())
                .expect("getting the metadata on a entry shouldn't fail")
        );

        table
            .assert_meta(&meta)
            .expect("asserting metadata shouldn't fail");
        assert_eq!(
            Some(&meta),
            table
                .get_meta(&meta.key())
                .expect("getting the metadata on a entry shouldn't fail")
                .as_ref()
        );
    }

    #[test]
    /// all Meta for an Entry can be retrieved with meta_from_entry() and meta_from_request()
    fn meta_from() {
        let mut table = test_table();
        let entry = test_entry();
        let meta_a = test_meta_a();
        let meta_b = test_meta_b();
        let empty_vec: Vec<EntryMeta> = Vec::new();

        assert_eq!(
            empty_vec,
            table
                .metas_from_entry(&entry)
                .expect("getting the metadata on a entry shouldn't fail")
        );

        table
            .assert_meta(&meta_a)
            .expect("asserting metadata shouldn't fail");
        assert_eq!(
            vec![meta_a.clone()],
            table
                .metas_from_entry(&entry)
                .expect("getting the metadata on a entry shouldn't fail")
        );

        table
            .assert_meta(&meta_b.clone())
            .expect("asserting metadata shouldn't fail");
        assert_eq!(
            vec![meta_b.clone(), meta_a.clone()],
            table
                .metas_from_entry(&entry)
                .expect("getting the metadata on a entry shouldn't fail")
        );

        // test meta_from_request()
        assert_eq!(
            Some(meta_a.clone()),
            table
                .meta_from_request(entry.key(), &meta_a.attribute())
                .expect("getting the metadata on a entry shouldn't fail")
        );
        assert_eq!(
            Some(meta_b.clone()),
            table
                .meta_from_request(entry.key(), &meta_b.attribute())
                .expect("getting the metadata on a entry shouldn't fail")
        );
    }

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

    #[test]
    fn test_standard_suite() {
        standard_suite(&mut test_table());
    }

}
