use error::HolochainError;

use cas::{content::Address, memory::MemoryStorage, storage::ContentAddressableStorage};
use hash::HashString;
use hash_table::{entry::Entry, entry_meta::EntryMeta, HashTable};
use key::Key;
use std::collections::HashMap;

/// Struct implementing the HashTable Trait by storing the HashTable in memory
#[derive(Serialize, Debug, Clone, PartialEq, Default)]
pub struct MemTable {
    entry_storage: MemoryStorage,
    metas: HashMap<HashString, EntryMeta>,
}

impl MemTable {
    pub fn new(entry_storage: MemoryStorage) -> MemTable {
        MemTable {
            entry_storage: entry_storage,
            metas: HashMap::new(),
        }
    }
}

impl HashTable for MemTable {
    fn put_entry(&mut self, entry: &Entry) -> Result<(), HolochainError> {
        self.entry_storage.add(entry)
    }

    fn entry(&self, address: &Address) -> Result<Option<Entry>, HolochainError> {
        self.entry_storage.fetch(address)
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
            .filter(|&m| m.entry_hash() == &entry.key())
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

    use cas::memory::MemoryStorage;
    use hash_table::{memory::MemTable, test_util::standard_suite};

    pub fn test_table() -> MemTable {
        MemTable::new(MemoryStorage::new())
    }

    #[test]
    /// smoke test
    fn new() {
        test_table();
    }

    #[test]
    fn test_standard_suite() {
        standard_suite(&mut test_table());
    }

}
