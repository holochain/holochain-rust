use error::HolochainError;

use hash::HashString;
use hash_table::{entry::Entry, entry_meta::EntryMeta, HashTable};
use std::collections::HashMap;
use cas::content::Address;
use cas::content::AddressableContent;

/// Struct implementing the HashTable Trait by storing the HashTable in memory
#[derive(Serialize, Debug, Clone, PartialEq, Default)]
pub struct MemTable {
    entries: HashMap<HashString, Entry>,
    metas: HashMap<HashString, EntryMeta>,
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
        self.entries.insert(entry.address(), entry.clone());
        Ok(())
    }

    fn entry(&self, address: &Address) -> Result<Option<Entry>, HolochainError> {
        Ok(self.entries.get(address).cloned())
    }

    fn assert_meta(&mut self, meta: &EntryMeta) -> Result<(), HolochainError> {
        self.metas.insert(meta.address(), meta.clone());
        Ok(())
    }

    fn get_meta(&mut self, address: &Address) -> Result<Option<EntryMeta>, HolochainError> {
        Ok(self.metas.get(address).cloned())
    }

    /// Return all the Metas for an entry
    fn metas_from_entry(&mut self, entry: &Entry) -> Result<Vec<EntryMeta>, HolochainError> {
        let mut vec_meta = self
            .metas
            .values()
            .filter(|&m| m.entry_address() == &entry.address())
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

    use hash_table::{memory::MemTable, test_util::standard_suite};

    pub fn test_table() -> MemTable {
        MemTable::new()
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
