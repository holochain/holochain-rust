use holochain_core_types::{entry::Entry, error::HolochainError};
use holochain_locksmith::RwLock;
use holochain_persistence_api::cas::{
    content::{Address, AddressableContent},
    storage::ContentAddressableStorage,
};
use std::sync::Arc;

pub trait GetByAddress {
    fn content_storage(&self) -> Arc<RwLock<dyn ContentAddressableStorage>>;

    fn get(&self, address: &Address) -> Result<Option<Entry>, HolochainError> {
        if let Some(json) = (*self.content_storage().read().unwrap()).fetch(address)? {
            let entry = Entry::try_from_content(&json)?;
            Ok(Some(entry))
        } else {
            Ok(None) // no errors but entry is not in chain CAS
        }
    }
}
