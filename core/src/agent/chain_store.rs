use agent::chain_header::ChainHeader;
use cas::storage::ContentAddressableStorage;

#[derive(Debug, PartialEq, Clone)]
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

    pub fn iter(&self, top_chain_header: &ChainHeader) -> ChainStoreIterator<CAS> {
        ChainStoreIterator::new(self.content_storage.clone(), Some(top_chain_header.clone()))
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
        println!("foo");
        println!("{:?}", self.current);

        let previous = self.current.take();

        self.current = previous
            .as_ref()
            .and_then(|chain_header| chain_header.link())
            .as_ref()
            // @TODO should this panic?
            // @see https://github.com/holochain/holochain-rust/issues/146
            .and_then(|linked_chain_header_address| {
                println!("{:?}", linked_chain_header_address);
                self.content_storage.fetch(linked_chain_header_address).expect("failed to fetch from CAS")
            });
        println!("{:?}", self.current);
        println!("bar");
        previous
    }
}

#[cfg(test)]
pub mod tests {

    use agent::{
        chain_header::{tests::test_chain_header, ChainHeader},
        chain_store::ChainStore,
    };
    use cas::{
        content::AddressableContent, memory::MemoryStorage, storage::ContentAddressableStorage,
    };
    use hash_table::entry::tests::{test_entry, test_entry_type};

    pub fn test_chain_store() -> ChainStore<MemoryStorage> {
        ChainStore::new(MemoryStorage::new().expect("could not create new chain store"))
    }

    #[test]
    /// show Iterator implementation for chain store
    fn iterator() {
        let chain_store = test_chain_store();

        let chain_header_a = test_chain_header();
        let chain_header_b = ChainHeader::new(
            &test_entry_type(),
            &String::new(),
            Some(chain_header_a.address()),
            &test_entry().address(),
            &String::new(),
            None,
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
        for chain_header in chain_store.iter(&chain_header_b) {
            println!("{:?}", chain_header);
            found.push(chain_header);
        }
        assert_eq!(expected, found);
    }
}
