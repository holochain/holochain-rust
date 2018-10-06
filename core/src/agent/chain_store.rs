use cas::storage::ContentAddressableStorage;
use agent::chain_header::ChainHeader;

#[derive(Debug, PartialEq, Clone)]
pub struct ChainStore<CAS> where CAS: ContentAddressableStorage + Sized + Clone + PartialEq {
    // Storages holding local shard data
    content_storage: CAS,
}

impl<CAS> ChainStore<CAS> where
    CAS: ContentAddressableStorage + Sized + Clone + PartialEq {

    pub fn new(content_storage: CAS) -> Self {
        ChainStore {
            content_storage,
        }
    }

    pub fn content_storage(&self) -> CAS {
        self.content_storage.clone()
    }
}

pub struct ChainStoreIterator<CAS> where CAS: ContentAddressableStorage + Sized + Clone + PartialEq {
    content_storage: CAS,
    current: Option<ChainHeader>,
}

impl<CAS> ChainStoreIterator<CAS> where CAS: ContentAddressableStorage + Sized + Clone + PartialEq {
    #[allow(unknown_lints)]
    #[allow(needless_pass_by_value)]
    pub fn new(
        content_storage: CAS,
        current: &Option<ChainHeader>,
    ) -> ChainStoreIterator<CAS> {
        ChainStoreIterator {
            content_storage: content_storage.clone(),
            current: current.clone(),
        }
    }
}

impl<CAS> Iterator for ChainStoreIterator<CAS> where CAS: ContentAddressableStorage + Sized + Clone + PartialEq {
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

#[cfg(test)]
pub mod tests {

    pub fn test_chain_store() -> ChainStore<MemoryStorage> {

    }
}
