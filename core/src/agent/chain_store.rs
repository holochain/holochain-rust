use holochain_core_types::{
    cas::{
        content::{Address, AddressableContent},
        storage::ContentAddressableStorage,
    },
    chain_header::ChainHeader,
    entry::entry_type::EntryType,
};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub struct ChainStore {
    // Storages holding local shard data
    content_storage: Arc<RwLock<dyn ContentAddressableStorage>>,
}

impl PartialEq for ChainStore {
    fn eq(&self, other: &ChainStore) -> bool {
        let storage_lock = &self.content_storage.clone();
        let storage = &*storage_lock.read().unwrap();
        let other_storage_lock = &other.content_storage.clone();
        let other_storage = &*other_storage_lock.read().unwrap();
        storage.get_id() == other_storage.get_id()
    }
}

impl ChainStore {
    pub fn new(content_storage: Arc<RwLock<dyn ContentAddressableStorage>>) -> Self {
        ChainStore { content_storage }
    }

    pub fn content_storage(&self) -> Arc<RwLock<dyn ContentAddressableStorage>> {
        self.content_storage.clone()
    }

    pub fn iter(&self, start_chain_header: &Option<ChainHeader>) -> ChainStoreIterator {
        ChainStoreIterator::new(self.content_storage.clone(), start_chain_header.clone())
    }

    pub fn iter_type(
        &self,
        start_chain_header: &Option<ChainHeader>,
        entry_type: &EntryType,
    ) -> ChainStoreTypeIterator {
        ChainStoreTypeIterator::new(
            self.content_storage.clone(),
            self.iter(start_chain_header)
                .find(|chain_header| chain_header.entry_type() == entry_type),
        )
    }

    pub fn query(
        &self,
        start_chain_header: &Option<ChainHeader>,
        entry_type: &EntryType,
        start: u32,
        limit: u32,
    ) -> Vec<Address> {
        let base_iter = self
            .iter_type(start_chain_header, entry_type)
            .map(|header| header.entry_address().clone())
            .skip(start as usize);

        if limit > 0 {
            base_iter.take(limit as usize).collect()
        } else {
            base_iter.collect()
        }
    }
}

pub struct ChainStoreIterator {
    content_storage: Arc<RwLock<dyn ContentAddressableStorage>>,
    current: Option<ChainHeader>,
}

impl ChainStoreIterator {
    pub fn new(
        content_storage: Arc<RwLock<dyn ContentAddressableStorage>>,
        current: Option<ChainHeader>,
    ) -> ChainStoreIterator {
        ChainStoreIterator {
            content_storage,
            current,
        }
    }
}

impl Iterator for ChainStoreIterator {
    type Item = ChainHeader;

    /// May panic if there is an underlying error in the table
    fn next(&mut self) -> Option<ChainHeader> {
        let previous = self.current.take();
        let storage = &self.content_storage.clone();
        self.current = previous
            .as_ref()
            .and_then(|chain_header| chain_header.link())
            .as_ref()
            // @TODO should this panic?
            // @see https://github.com/holochain/holochain-rust/issues/146
            .and_then(|linked_chain_header_address| {
                storage
                    .read()
                    .unwrap()
                    .fetch(linked_chain_header_address)
                    .expect("failed to fetch from CAS")
                    .map(|content| {
                        ChainHeader::try_from_content(&content)
                            .expect("failed to load ChainHeader from Content")
                    })
            });
        previous
    }
}

pub struct ChainStoreTypeIterator {
    content_storage: Arc<RwLock<dyn ContentAddressableStorage>>,
    current: Option<ChainHeader>,
}

impl ChainStoreTypeIterator {
    pub fn new(
        content_storage: Arc<RwLock<dyn ContentAddressableStorage>>,
        current: Option<ChainHeader>,
    ) -> ChainStoreTypeIterator {
        ChainStoreTypeIterator {
            content_storage,
            current,
        }
    }
}

impl Iterator for ChainStoreTypeIterator {
    type Item = ChainHeader;

    /// May panic if there is an underlying error in the table
    fn next(&mut self) -> Option<ChainHeader> {
        let previous = self.current.take();
        let storage = &self.content_storage.clone();
        self.current = previous
            .as_ref()
            .and_then(|chain_header| chain_header.link_same_type())
            .as_ref()
            // @TODO should this panic?
            // @see https://github.com/holochain/holochain-rust/issues/146
            .and_then(|linked_chain_header_address| {
                (*storage.read().unwrap())
                    .fetch(linked_chain_header_address)
                    .expect("failed to fetch from CAS")
                    .map(|content| {
                        ChainHeader::try_from_content(&content)
                            .expect("failed to load ChainHeader from Content")
                    })
            });
        previous
    }
}

#[cfg(test)]
pub mod tests {
    extern crate tempfile;
    use self::tempfile::tempdir;
    use crate::agent::chain_store::ChainStore;
    use holochain_cas_implementations::cas::file::FilesystemStorage;
    use holochain_core_types::{
        cas::content::AddressableContent,
        chain_header::{test_chain_header, test_sources, ChainHeader},
        entry::{test_entry, test_entry_b, test_entry_c},
        signature::{test_signature_b, test_signature_c, test_signatures},
        time::test_iso_8601,
    };
    use std::sync::{Arc, RwLock};

    pub fn test_chain_store() -> ChainStore {
        ChainStore::new(Arc::new(RwLock::new(
            FilesystemStorage::new(tempdir().unwrap().path().to_str().unwrap())
                .expect("could not create chain store"),
        )))
    }

    #[test]
    /// show Iterator implementation for chain store
    fn iterator_test() {
        let chain_store = test_chain_store();

        let entry = test_entry_b();
        let chain_header_a = test_chain_header();
        let chain_header_b = ChainHeader::new(
            &entry.entry_type(),
            &entry.address(),
            &test_sources(),
            &vec![test_signature_b()],
            &Some(chain_header_a.address()),
            &None,
            &None,
            &test_iso_8601(),
        );

        let storage = chain_store.content_storage.clone();
        (*storage.write().unwrap())
            .add(&chain_header_a)
            .expect("could not add header to cas");
        (*storage.write().unwrap())
            .add(&chain_header_b)
            .expect("could not add header to cas");

        let expected = vec![chain_header_b.clone(), chain_header_a.clone()];
        let mut found = vec![];
        for chain_header in chain_store.iter(&Some(chain_header_b)) {
            found.push(chain_header);
        }
        assert_eq!(expected, found);

        let expected = vec![chain_header_a.clone()];
        let mut found = vec![];
        for chain_header in chain_store.iter(&Some(chain_header_a)) {
            found.push(chain_header);
        }
        assert_eq!(expected, found);
    }

    #[test]
    /// show entry typed Iterator implementation for chain store
    fn type_iterator_test() {
        let chain_store = test_chain_store();

        let chain_header_a = test_chain_header();
        // b has a different type to a
        let entry_b = test_entry_b();
        let chain_header_b = ChainHeader::new(
            &entry_b.entry_type(),
            &entry_b.address(),
            &test_sources(),
            &test_signatures(),
            &Some(chain_header_a.address()),
            &None,
            &None,
            &test_iso_8601(),
        );
        // c has same type as a
        let entry_c = test_entry();
        let chain_header_c = ChainHeader::new(
            &entry_c.entry_type(),
            &entry_c.address(),
            &test_sources(),
            &test_signatures(),
            &Some(chain_header_b.address()),
            &Some(chain_header_a.address()),
            &None,
            &test_iso_8601(),
        );

        for chain_header in vec![&chain_header_a, &chain_header_b, &chain_header_c] {
            let storage = chain_store.content_storage.clone();
            (*storage.write().unwrap())
                .add(chain_header)
                .expect("could not add header to cas");
        }

        let expected = vec![chain_header_c.clone(), chain_header_a.clone()];
        let mut found = vec![];
        for chain_header in
            chain_store.iter_type(&Some(chain_header_c.clone()), &chain_header_c.entry_type())
        {
            found.push(chain_header);
        }
        assert_eq!(expected, found);

        let expected = vec![chain_header_a.clone()];
        let mut found = vec![];
        for chain_header in
            chain_store.iter_type(&Some(chain_header_b.clone()), &chain_header_c.entry_type())
        {
            found.push(chain_header);
        }
        assert_eq!(expected, found);

        let expected = vec![chain_header_b.clone()];
        let mut found = vec![];
        for chain_header in
            chain_store.iter_type(&Some(chain_header_c.clone()), &chain_header_b.entry_type())
        {
            found.push(chain_header);
        }
        assert_eq!(expected, found);

        let expected = vec![chain_header_b.clone()];
        let mut found = vec![];
        for chain_header in
            chain_store.iter_type(&Some(chain_header_b.clone()), &chain_header_b.entry_type())
        {
            found.push(chain_header);
        }
        assert_eq!(expected, found);
    }

    #[test]
    /// show query() implementation
    fn query_test() {
        let chain_store = test_chain_store();

        let chain_header_a = test_chain_header();
        let entry = test_entry_b();
        let chain_header_b = ChainHeader::new(
            &entry.entry_type(),
            &entry.address(),
            &test_sources(),
            &vec![test_signature_b()],
            &Some(chain_header_a.address()),
            &None,
            &None,
            &test_iso_8601(),
        );
        let entry = test_entry_c();
        let chain_header_c = ChainHeader::new(
            &entry.entry_type(),
            &entry.address(),
            &test_sources(),
            &vec![test_signature_c()],
            &Some(chain_header_b.address()),
            &Some(chain_header_b.address()),
            &None,
            &test_iso_8601(),
        );

        let storage = chain_store.content_storage.clone();
        (*storage.write().unwrap())
            .add(&chain_header_a)
            .expect("could not add header to cas");
        (*storage.write().unwrap())
            .add(&chain_header_b)
            .expect("could not add header to cas");
        (*storage.write().unwrap())
            .add(&chain_header_c)
            .expect("could not add header to cas");

        let found = chain_store.query(&Some(chain_header_c.clone()), &entry.entry_type(), 0, 0);
        let expected = vec![
            chain_header_c.entry_address().clone(),
            chain_header_b.entry_address().clone(),
        ];
        assert_eq!(expected, found);

        let found = chain_store.query(&Some(chain_header_c.clone()), &entry.entry_type(), 0, 1);
        let expected = vec![chain_header_c.entry_address().clone()];
        assert_eq!(expected, found);
    }

}
