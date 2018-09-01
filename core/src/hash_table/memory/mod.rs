use std::collections::HashMap;

use error::HolochainError;
use hash_table::{pair::Pair, pair_meta::PairMeta};
use hash_table::HashTable;
use key::Key;

/// Struct implementing the HashTable Trait by storing the HashTable in memory
#[derive(Serialize, Debug, Clone, PartialEq, Default)]
pub struct MemTable {
    pairs: HashMap<String, Pair>,
    meta: HashMap<String, PairMeta>,
}

impl MemTable {
    pub fn new() -> MemTable {
        MemTable {
            pairs: HashMap::new(),
            meta: HashMap::new(),
        }
    }
}

impl HashTable for MemTable {
    fn commit_pair(&mut self, pair: &Pair) -> Result<(), HolochainError> {
        self.pairs.insert(pair.key(), pair.clone());
        Ok(())
    }

    fn pair(&self, key: &str) -> Result<Option<Pair>, HolochainError> {
        Ok(self.pairs.get(key).cloned())
    }

    fn assert_pair_meta(&mut self, meta: &PairMeta) -> Result<(), HolochainError> {
        self.meta.insert(meta.key(), meta.clone());
        Ok(())
    }

    fn pair_meta(&mut self, key: &str) -> Result<Option<PairMeta>, HolochainError> {
        Ok(self.meta.get(key).cloned())
    }

    fn metas_for_pair(&mut self, pair: &Pair) -> Result<Vec<PairMeta>, HolochainError> {
        let mut metas = self
            .meta
            .values()
            .filter(|&m| m.pair_hash() == pair.key())
            .cloned()
            .collect::<Vec<PairMeta>>();
        // @TODO should this be sorted at all at this point?
        // @see https://github.com/holochain/holochain-rust/issues/144
        metas.sort();
        Ok(metas)
    }
}

#[cfg(test)]
pub mod tests {

    use hash_table::{
        memory::MemTable,
        test_util::standard_suite,
    };

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
