use holochain_core_types::{
    cas::{content::Address, storage::ContentAddressableStorage},
    chain_header::ChainHeader,
    entry_type::EntryType,
};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ChainStore<CAS>
where
    CAS: ContentAddressableStorage + Sized + Clone + PartialEq,
{
    // Storages holding local shard data
    content_storage: CAS,
}

impl<CAS> ChainStore<CAS>
where
    CAS: ContentAddressableStorage + Sized + Clone + PartialEq,
{
    pub fn new(content_storage: CAS) -> Self {
        ChainStore { content_storage }
    }

    pub fn content_storage(&self) -> CAS {
        self.content_storage.clone()
    }

    pub fn iter(&self, start_chain_header: &Option<ChainHeader>) -> ChainStoreIterator<CAS> {
        ChainStoreIterator::new(self.content_storage.clone(), start_chain_header.clone())
    }

    pub fn iter_type(
        &self,
        start_chain_header: &Option<ChainHeader>,
        entry_type: &EntryType,
    ) -> ChainStoreTypeIterator<CAS> {
        ChainStoreTypeIterator::new(
            self.content_storage.clone(),
            self.iter(start_chain_header)
                .find(|chain_header| chain_header.entry_type() == entry_type),
        )
    }

    pub fn query(
        &self,
        start_chain_header: &Option<ChainHeader>,
        entry_type: EntryType,
        limit: u32,
    ) -> Vec<Address> {
        let mut result: Vec<Address> = Vec::new();
        for header in self.iter_type(start_chain_header, &entry_type) {
            result.push(header.entry_address().clone());
            if limit != 0 && result.len() as u32 >= limit {
                break;
            }
        }
        result
    }
}

pub struct ChainStoreIterator<CAS>
where
    CAS: ContentAddressableStorage + Sized + Clone + PartialEq,
{
    content_storage: CAS,
    current: Option<ChainHeader>,
}

impl<CAS> ChainStoreIterator<CAS>
where
    CAS: ContentAddressableStorage + Sized + Clone + PartialEq,
{
    #[allow(unknown_lints)]
    #[allow(needless_pass_by_value)]
    pub fn new(content_storage: CAS, current: Option<ChainHeader>) -> ChainStoreIterator<CAS> {
        ChainStoreIterator {
            content_storage,
            current,
        }
    }
}

impl<CAS> Iterator for ChainStoreIterator<CAS>
where
    CAS: ContentAddressableStorage + Sized + Clone + PartialEq,
{
    type Item = ChainHeader;

    /// May panic if there is an underlying error in the table
    fn next(&mut self) -> Option<ChainHeader> {
        let previous = self.current.take();

        self.current = previous
            .as_ref()
            .and_then(|chain_header| chain_header.link())
            .as_ref()
            // @TODO should this panic?
            // @see https://github.com/holochain/holochain-rust/issues/146
            .and_then(|linked_chain_header_address| {
                self.content_storage.fetch(linked_chain_header_address).expect("failed to fetch from CAS")
            });
        previous
    }
}

pub struct ChainStoreTypeIterator<CAS>
where
    CAS: ContentAddressableStorage + Sized + Clone + PartialEq,
{
    content_storage: CAS,
    current: Option<ChainHeader>,
}

impl<CAS> ChainStoreTypeIterator<CAS>
where
    CAS: ContentAddressableStorage + Sized + Clone + PartialEq,
{
    #[allow(unknown_lints)]
    #[allow(needless_pass_by_value)]
    pub fn new(content_storage: CAS, current: Option<ChainHeader>) -> ChainStoreTypeIterator<CAS> {
        ChainStoreTypeIterator {
            content_storage,
            current,
        }
    }
}

impl<CAS> Iterator for ChainStoreTypeIterator<CAS>
where
    CAS: ContentAddressableStorage + Sized + Clone + PartialEq,
{
    type Item = ChainHeader;

    /// May panic if there is an underlying error in the table
    fn next(&mut self) -> Option<ChainHeader> {
        let previous = self.current.take();

        self.current = previous
            .as_ref()
            .and_then(|chain_header| chain_header.link_same_type())
            .as_ref()
            // @TODO should this panic?
            // @see https://github.com/holochain/holochain-rust/issues/146
            .and_then(|linked_chain_header_address| {
                self.content_storage.fetch(linked_chain_header_address).expect("failed to fetch from CAS")
            });
        previous
    }
}

#[cfg(test)]
pub mod tests {
    extern crate tempfile;
    use self::tempfile::tempdir;
    use agent::chain_store::ChainStore;
    use holochain_cas_implementations::cas::file::FilesystemStorage;
    use holochain_core_types::{
        cas::{content::AddressableContent, storage::ContentAddressableStorage},
        chain_header::{test_chain_header, ChainHeader},
        entry::{test_entry, test_entry_b, test_entry_c},
        signature::{test_signature, test_signature_b, test_signature_c},
        time::test_iso_8601,
    };

    pub fn test_chain_store() -> ChainStore<FilesystemStorage> {
        ChainStore::new(
            FilesystemStorage::new(tempdir().unwrap().path().to_str().unwrap())
                .expect("could not create new chain store"),
        )
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
            &test_signature_b(),
            &Some(chain_header_a.address()),
            &None,
            &test_iso_8601(),
        );

        chain_store
            .content_storage()
            .add(&chain_header_a)
            .expect("could not add header to cas");
        chain_store
            .content_storage()
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
            &test_signature(),
            &Some(chain_header_a.address()),
            &None,
            &test_iso_8601(),
        );
        // c has same type as a
        let entry_c = test_entry();
        let chain_header_c = ChainHeader::new(
            &entry_c.entry_type(),
            &entry_c.address(),
            &test_signature(),
            &Some(chain_header_b.address()),
            &Some(chain_header_a.address()),
            &test_iso_8601(),
        );

        for chain_header in vec![&chain_header_a, &chain_header_b, &chain_header_c] {
            chain_store
                .content_storage()
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
            &test_signature_b(),
            &Some(chain_header_a.address()),
            &None,
            &test_iso_8601(),
        );
        let entry = test_entry_c();
        let chain_header_c = ChainHeader::new(
            &entry.entry_type(),
            &entry.address(),
            &test_signature_c(),
            &Some(chain_header_b.address()),
            &Some(chain_header_b.address()),
            &test_iso_8601(),
        );

        chain_store
            .content_storage()
            .add(&chain_header_a)
            .expect("could not add header to cas");
        chain_store
            .content_storage()
            .add(&chain_header_b)
            .expect("could not add header to cas");
        chain_store
            .content_storage()
            .add(&chain_header_c)
            .expect("could not add header to cas");

        let found = chain_store.query(
            &Some(chain_header_c.clone()),
            entry.entry_type().to_owned(),
            0,
        );
        let expected = vec![
            chain_header_c.entry_address().clone(),
            chain_header_b.entry_address().clone(),
        ];
        assert_eq!(expected, found);

        let found = chain_store.query(
            &Some(chain_header_c.clone()),
            entry.entry_type().to_owned(),
            1,
        );
        let expected = vec![chain_header_c.entry_address().clone()];
        assert_eq!(expected, found);
    }

}
