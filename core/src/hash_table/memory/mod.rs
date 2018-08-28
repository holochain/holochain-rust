use std::collections::HashMap;

use error::HolochainError;

use agent::keys::Keys;
use hash_table::{
    pair::Pair,
    pair_meta::PairMeta,
    status::{CRUDStatus, LINK_NAME, STATUS_NAME},
    HashTable,
};

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
    fn setup(&mut self) -> Result<(), HolochainError> {
        Ok(())
    }

    fn teardown(&mut self) -> Result<(), HolochainError> {
        Ok(())
    }

    fn commit(&mut self, pair: &Pair) -> Result<(), HolochainError> {
        self.pairs.insert(pair.key(), pair.clone());
        Ok(())
    }

    fn pair(&self, key: &str) -> Result<Option<Pair>, HolochainError> {
        Ok(self.pairs.get(key).cloned())
    }

    fn modify(
        &mut self,
        keys: &Keys,
        old_pair: &Pair,
        new_pair: &Pair,
    ) -> Result<(), HolochainError> {
        self.commit(new_pair)?;

        // @TODO what if meta fails when commit succeeds?
        // @see https://github.com/holochain/holochain-rust/issues/142
        self.assert_meta(&PairMeta::new(
            keys,
            &old_pair,
            STATUS_NAME,
            &CRUDStatus::MODIFIED.bits().to_string(),
        ))?;

        // @TODO what if meta fails when commit succeeds?
        // @see https://github.com/holochain/holochain-rust/issues/142
        self.assert_meta(&PairMeta::new(keys, &old_pair, LINK_NAME, &new_pair.key()))
    }

    fn retract(&mut self, keys: &Keys, pair: &Pair) -> Result<(), HolochainError> {
        self.assert_meta(&PairMeta::new(
            keys,
            &pair,
            STATUS_NAME,
            &CRUDStatus::DELETED.bits().to_string(),
        ))
    }

    fn assert_meta(&mut self, meta: &PairMeta) -> Result<(), HolochainError> {
        self.meta.insert(meta.key(), meta.clone());
        Ok(())
    }

    fn get_meta(&mut self, key: &str) -> Result<Option<PairMeta>, HolochainError> {
        Ok(self.meta.get(key).cloned())
    }

    fn get_pair_meta(&mut self, pair: &Pair) -> Result<Vec<PairMeta>, HolochainError> {
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

    use agent::keys::tests::test_keys;
    use hash_table::{
        memory::MemTable,
        pair::tests::{test_pair, test_pair_a, test_pair_b},
        pair_meta::{
            tests::{test_pair_meta, test_pair_meta_a, test_pair_meta_b},
            PairMeta,
        },
        status::{CRUDStatus, LINK_NAME, STATUS_NAME},
        HashTable,
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
        let mut table = test_table();
        let pair = test_pair();
        table.commit(&pair).expect("should be able to commit valid pair");
        assert_eq!(table.pair(&pair.key()), Ok(Some(pair)));
    }

    #[test]
    /// Pairs can be modified through table.modify()
    fn modify() {
        let mut ht = test_table();
        let p1 = test_pair_a();
        let p2 = test_pair_b();

        ht.commit(&p1).expect("should be able to commit valid pair");
        ht.modify(&test_keys(), &p1, &p2)
            .expect("should be able to edit with valid pair");

        assert_eq!(
            vec![
                PairMeta::new(&test_keys(), &p1, LINK_NAME, &p2.key()),
                PairMeta::new(
                    &test_keys(),
                    &p1,
                    STATUS_NAME,
                    &CRUDStatus::MODIFIED.bits().to_string(),
                ),
            ],
            ht.get_pair_meta(&p1)
                .expect("getting the metadata on a pair shouldn't fail")
        );

        let empty_vec: Vec<PairMeta> = Vec::new();
        assert_eq!(
            empty_vec,
            ht.get_pair_meta(&p2)
                .expect("getting the metadata on a pair shouldn't fail")
        );
    }

    #[test]
    /// Pairs can be retracted through table.retract()
    fn retract() {
        let mut ht = test_table();
        let p = test_pair();
        let empty_vec: Vec<PairMeta> = Vec::new();

        ht.commit(&p).expect("should be able to commit valid pair");
        assert_eq!(
            empty_vec,
            ht.get_pair_meta(&p)
                .expect("getting the metadata on a pair shouldn't fail")
        );

        ht.retract(&test_keys(), &p)
            .expect("should be able to retract");
        assert_eq!(
            vec![PairMeta::new(
                &test_keys(),
                &p,
                STATUS_NAME,
                &CRUDStatus::DELETED.bits().to_string(),
            )],
            ht.get_pair_meta(&p)
                .expect("getting the metadata on a pair shouldn't fail"),
        );
    }

    #[test]
    /// PairMeta can round trip through table.assert_meta() and table.get_meta()
    fn meta_round_trip() {
        let mut table = test_table();
        let pair_meta = test_pair_meta();

        assert_eq!(
            None,
            table.get_meta(&pair_meta.key())
                .expect("getting the metadata on a pair shouldn't fail")
        );

        table.assert_meta(&pair_meta)
            .expect("asserting metadata shouldn't fail");
        assert_eq!(
            Some(&pair_meta),
            table.get_meta(&pair_meta.key())
                .expect("getting the metadata on a pair shouldn't fail")
                .as_ref()
        );
    }

    #[test]
    /// all PairMeta for a Pair can be retrieved with pair_meta
    fn get_pair_meta() {
        let mut table = test_table();
        let pair = test_pair();
        let pair_meta_a = test_pair_meta_a();
        let pair_meta_b = test_pair_meta_b();
        let empty_vec: Vec<PairMeta> = Vec::new();

        assert_eq!(
            empty_vec,
            table.get_pair_meta(&pair)
                .expect("getting the metadata on a pair shouldn't fail")
        );

        table.assert_meta(&pair_meta_a)
            .expect("asserting metadata shouldn't fail");
        assert_eq!(
            vec![pair_meta_a.clone()],
            table.get_pair_meta(&pair)
                .expect("getting the metadata on a pair shouldn't fail")
        );

        table.assert_meta(&pair_meta_b.clone())
            .expect("asserting metadata shouldn't fail");
        assert_eq!(
            vec![pair_meta_b, pair_meta_a],
            table.get_pair_meta(&pair)
                .expect("getting the metadata on a pair shouldn't fail")
        );
    }
}
