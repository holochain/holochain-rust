use holochain_core_types::{eav::Attribute, entry::Entry, error::HcResult};
use holochain_persistence_api::{
    cas::content::{Address, AddressableContent, Content},
    txn::{Cursor, CursorRwDyn},
};

pub trait GetContent {
    /// Return the content at this address, do not attempt to convert to an entry
    fn get_raw(&self, address: &Address) -> HcResult<Option<Content>>;

    /// Get an entry from this storage
    fn get(&self, address: &Address) -> HcResult<Option<Entry>> {
        if let Some(json) = self.get_raw(address)? {
            let entry = Entry::try_from_content(&json)?;
            Ok(Some(entry))
        } else {
            Ok(None) // no errors but entry is not in chain CAS
        }
    }

    fn contains(&self, address: &Address) -> HcResult<bool> {
        Ok(self.get_raw(address)?.is_some())
    }
}

impl GetContent for dyn Cursor<Attribute> {
    fn get_raw(&self, address: &Address) -> HcResult<Option<Content>> {
        Ok(self.fetch(address)?)
    }
}

impl GetContent for dyn CursorRwDyn<Attribute> {
    fn get_raw(&self, address: &Address) -> HcResult<Option<Content>> {
        Ok(self.fetch(address)?)
    }
}
