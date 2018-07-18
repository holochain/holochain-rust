use std::collections::HashMap;

use error::HolochainError;

use agent::keys::Keys;
use hash_table::{
    pair::Pair, pair_meta::PairMeta, status::{CRUDStatus, LINK_NAME, STATUS_NAME}, HashTable,
};

#[derive(Serialize, Debug, Clone, PartialEq)]
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

    fn get(&self, key: &str) -> Result<Option<Pair>, HolochainError> {
        Ok(self.pairs.get(key.into()).and_then(|p| Some(p.clone())))
    }

    fn modify(
        &mut self,
        keys: &Keys,
        old_pair: &Pair,
        new_pair: &Pair,
    ) -> Result<(), HolochainError> {
        let result = self.commit(new_pair);
        if result.is_err() {
            return result;
        }

        // @TODO what if meta fails when commit succeeds?
        // @see https://github.com/holochain/holochain-rust/issues/142
        let result = self.assert_meta(&PairMeta::new(
            keys,
            &old_pair,
            STATUS_NAME,
            &CRUDStatus::MODIFIED.bits().to_string(),
        ));
        if result.is_err() {
            return result;
        }

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
        Ok(self.meta.get(key).and_then(|m| Some(m.clone())))
    }

    fn get_pair_meta(&mut self, pair: &Pair) -> Result<Vec<PairMeta>, HolochainError> {
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
        memory::MemTable, pair::tests::{test_pair, test_pair_a, test_pair_b},
        pair_meta::{
            tests::{test_pair_meta, test_pair_meta_a, test_pair_meta_b}, PairMeta,
        },
        status::{CRUDStatus, LINK_NAME, STATUS_NAME}, HashTable,
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
        let mut ht = test_table();
        let p = test_pair();
        ht.commit(&p).unwrap();
        assert_eq!(ht.get(&p.key()), Ok(Some(p)));
    }

    #[test]
    /// Pairs can be modified through table.modify()
    fn modify() {
        let mut ht = test_table();
        let p1 = test_pair_a();
        let p2 = test_pair_b();

        ht.commit(&p1).unwrap();
        ht.modify(&test_keys(), &p1, &p2).unwrap();

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
            ht.get_pair_meta(&p1).unwrap()
        );

        let empty_vec: Vec<PairMeta> = Vec::new();
        assert_eq!(empty_vec, ht.get_pair_meta(&p2).unwrap());
    }

    #[test]
    /// Pairs can be retracted through table.retract()
    fn retract() {
        let mut ht = test_table();
        let p = test_pair();
        let empty_vec: Vec<PairMeta> = Vec::new();

        ht.commit(&p).unwrap();
        assert_eq!(empty_vec, ht.get_pair_meta(&p).unwrap());

        ht.retract(&test_keys(), &p).unwrap();
        assert_eq!(
            vec![PairMeta::new(
                &test_keys(),
                &p,
                STATUS_NAME,
                &CRUDStatus::DELETED.bits().to_string(),
            )],
            ht.get_pair_meta(&p).unwrap(),
        );
    }

    #[test]
    /// PairMeta can round trip through table.assert_meta() and table.get_meta()
    fn meta_round_trip() {
        let mut ht = test_table();
        let m = test_pair_meta();

        assert_eq!(None, ht.get_meta(&m.key()).unwrap());

        ht.assert_meta(&m).unwrap();
        assert_eq!(Some(m.clone()), ht.get_meta(&m.key()).unwrap());
    }

    #[test]
    /// all PairMeta for a Pair can be retrieved with get_pair_meta
    fn get_pair_meta() {
        let mut ht = test_table();
        let p = test_pair();
        let m1 = test_pair_meta_a();
        let m2 = test_pair_meta_b();
        let empty_vec: Vec<PairMeta> = Vec::new();

        assert_eq!(empty_vec, ht.get_pair_meta(&p).unwrap());

        ht.assert_meta(&m1).unwrap();
        assert_eq!(vec![m1.clone()], ht.get_pair_meta(&p).unwrap());

        ht.assert_meta(&m2).unwrap();
        assert_eq!(vec![m2.clone(), m1.clone()], ht.get_pair_meta(&p).unwrap());
    }
}
