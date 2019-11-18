use holochain_core_types::{entry::Entry, error::HolochainError};
use holochain_persistence_api::cas::content::{Address, AddressableContent, Content};

pub trait GetContent {
    /// Return the content at this addres, do not attempt to convert to an entry
    fn get_raw(&self, address: &Address) -> Result<Option<Content>, HolochainError>;

    /// Get an entry from this storage
    fn get(&self, address: &Address) -> Result<Option<Entry>, HolochainError> {
        if let Some(json) = self.get_raw(address)? {
            let entry = Entry::try_from_content(&json)?;
            Ok(Some(entry))
        } else {
            Ok(None) // no errors but entry is not in chain CAS
        }
    }

    fn contains(&self, address: &Address) -> Result<bool, HolochainError> {
        Ok(self.get_raw(address)?.is_some())
    }
}

pub trait AddContent {
    fn add<T: AddressableContent>(&self, content: &T) -> Result<(), HolochainError>;
}
