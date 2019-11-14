use holochain_core_types::{entry::Entry, error::HolochainError};
use holochain_locksmith::RwLock;
use holochain_persistence_api::cas::{
    content::{Address, AddressableContent, Content},
    storage::ContentAddressableStorage,
};
use std::sync::Arc;

pub trait ContentStore {
    fn content_storage(&self) -> Arc<RwLock<dyn ContentAddressableStorage>>;
}

pub trait GetContent: ContentStore {
    /// Get an entry from this storage
    fn get(&self, address: &Address) -> Result<Option<Entry>, HolochainError> {
        if let Some(json) = self.get_raw(address)? {
            let entry = Entry::try_from_content(&json)?;
            Ok(Some(entry))
        } else {
            Ok(None) // no errors but entry is not in chain CAS
        }
    }

    /// Return the content at this addres, do not attempt to convert to an entry
    fn get_raw(&self, address: &Address) -> Result<Option<Content>, HolochainError> {
        Ok((*self.content_storage().read().unwrap()).fetch(address)?)
    }

    fn contains(&self, address: &Address) -> Result<bool, HolochainError> {
        Ok(self.get_raw(address)?.is_some())
    }
}

pub trait AddContent: ContentStore {
    fn add<T: AddressableContent>(&self, content: &T) -> Result<(), HolochainError> {
        (*self.content_storage().write().unwrap())
            .add(content)
            .map_err(|e| e.into())
    }
}
