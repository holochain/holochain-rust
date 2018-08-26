use std::collections::HashMap;

use error::HolochainError;

use hash_table::{
    pair::Pair,
    pair_meta::PairMeta,
    HashTable,
};
use key::Key;

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

    fn all_metas_for_pair(&mut self, pair: &Pair) -> Result<Vec<PairMeta>, HolochainError> {
        let mut metas = self
            .meta
            .values()
            .filter(|&m| m.pair() == pair.key())
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
        HashTable,
    };
    use hash_table::test_util::test_pair_round_trip;
    use hash_table::test_util::test_modify_pair;
    use hash_table::test_util::test_retract_pair;
    use hash_table::test_util::test_meta_round_trip;
    use hash_table::test_util::test_all_metas_for_pair;

    pub fn test_table() -> MemTable {
        MemTable::new()
    }

    #[test]
    /// smoke test
    fn new() {
        test_table();
    }

    #[test]
    /// tests for ht.setup()
    fn setup() {
        let mut ht = test_table();
        assert_eq!(Ok(()), ht.setup());
    }

    #[test]
    /// tests for ht.teardown()
    fn teardown() {
        let mut ht = test_table();
        assert_eq!(Ok(()), ht.teardown());
    }

    #[test]
    /// Pairs can round trip through table.commit() and table.get()
    fn pair_round_trip() {
        test_pair_round_trip(&mut test_table());
    }

    #[test]
    /// Pairs can be modified through table.modify()
    fn modify_pair() {
        let mut table = test_table();
        test_modify_pair(&mut table);
    }

    #[test]
    /// Pairs can be retracted through table.retract()
    fn retract_pair() {
        let mut table = test_table();
        test_retract_pair(&mut table);
    }

    #[test]
    /// PairMeta can round trip through table.assert_meta() and table.get_meta()
    fn meta_round_trip() {
        let mut table = test_table();
        test_meta_round_trip(&mut table);
    }

    #[test]
    /// all PairMeta for a Pair can be retrieved with all_metas_for_pair
    fn all_metas_for_pair() {
        let mut table = test_table();
        test_all_metas_for_pair(&mut table);
    }
}
