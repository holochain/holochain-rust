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

    use agent::keys::tests::test_keys;
    use hash_table::{
        memory::MemTable,
        pair::tests::{test_pair},
        pair_meta::{
            tests::{test_pair_meta, test_pair_meta_a, test_pair_meta_b},
            PairMeta,
        },
        status::{CRUDStatus, STATUS_NAME},
        HashTable,
    };
    use hash_table::test_util::test_round_trip;
    use hash_table::test_util::test_modify_pair;
    use key::Key;

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
        test_round_trip(&mut test_table());
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
        let pair = test_pair();
        let empty_vec: Vec<PairMeta> = Vec::new();

        table.commit_pair(&pair).expect("should be able to commit valid pair");
        assert_eq!(
            empty_vec,
            table.all_metas_for_pair(&pair)
                .expect("getting the metadata on a pair shouldn't fail")
        );

        table.retract_pair(&test_keys(), &pair)
            .expect("should be able to retract");
        assert_eq!(
            vec![PairMeta::new(
                &test_keys(),
                &pair,
                STATUS_NAME,
                &CRUDStatus::DELETED.bits().to_string(),
            )],
            table.all_metas_for_pair(&pair)
                .expect("getting the metadata on a pair shouldn't fail"),
        );
    }

    #[test]
    /// PairMeta can round trip through table.assert_meta() and table.get_meta()
    fn meta_round_trip() {
        let mut table = test_table();
        let meta = test_pair_meta();

        assert_eq!(
            None,
            table.pair_meta(&meta.key())
                .expect("getting the metadata on a pair shouldn't fail")
        );

        table.assert_pair_meta(&meta)
            .expect("asserting metadata shouldn't fail");
        assert_eq!(
            Some(&meta),
            table.pair_meta(&meta.key())
                .expect("getting the metadata on a pair shouldn't fail")
                .as_ref()
        );
    }

    #[test]
    /// all PairMeta for a Pair can be retrieved with all_metas_for_pair
    fn all_metas_for_pair() {
        let mut table = test_table();
        let pair = test_pair();
        let meta_a = test_pair_meta_a();
        let meta_b = test_pair_meta_b();
        let empty_vec: Vec<PairMeta> = Vec::new();

        assert_eq!(
            empty_vec,
            table.all_metas_for_pair(&pair)
                .expect("getting the metadata on a pair shouldn't fail")
        );

        table.assert_pair_meta(&meta_a)
            .expect("asserting metadata shouldn't fail");
        assert_eq!(
            vec![meta_a.clone()],
            table.all_metas_for_pair(&pair)
                .expect("getting the metadata on a pair shouldn't fail")
        );

        table.assert_pair_meta(&meta_b)
            .expect("asserting metadata shouldn't fail");
        assert_eq!(
            vec![meta_b, meta_a],
            table.all_metas_for_pair(&pair)
                .expect("getting the metadata on a pair shouldn't fail")
        );
    }
}
