use std::collections::HashMap;

use error::HolochainError;

use agent::keys::Key;
use agent::keys::Keys;
use hash_table::status::CRUDStatus;
use hash_table::pair::Pair;
use hash_table::HashTable;
use hash_table::pair_meta::PairMeta;
use hash_table::status::STATUS_NAME;
use hash_table::status::LINK_NAME;

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct MemTable {
    pairs: HashMap<String, Pair>,
    meta: HashMap<String, PairMeta>,
}

impl MemTable {

    pub fn new() -> MemTable {

        MemTable{
            pairs: HashMap::new(),
            meta: HashMap::new(),
        }

    }

}

impl HashTable for MemTable {

    fn open(&mut self) -> Result<(), HolochainError> {
        Ok(())
    }

    fn close(&mut self) -> Result<(), HolochainError> {
        Ok(())
    }

    fn commit(&mut self, pair: &Pair) -> Result<(), HolochainError> {
        self.pairs.insert(pair.key(), pair.clone());
        Ok(())
    }

    fn get(&self, key: &str) -> Result<Option<Pair>, HolochainError> {
        Ok(self.pairs.get(key.into()).and_then(|p| Some(p.clone())))
    }

    fn modify(&mut self, old_pair: &Pair, new_pair: &Pair) -> Result<(), HolochainError> {
        let result = self.commit(new_pair);
        if result.is_err() {
            return result
        }

        // @TODO what if meta fails when commit succeeds?
        // @see https://github.com/holochain/holochain-rust/issues/142
        let result = self.assert_meta(
            &PairMeta::new(
                &Keys::new(&Key::new(), &Key::new(), ""),
                &old_pair,
                STATUS_NAME,
                &CRUDStatus::MODIFIED.bits().to_string(),
            )
        );
        if result.is_err() {
            return result
        }

        // @TODO what if meta fails when commit succeeds?
        // @see https://github.com/holochain/holochain-rust/issues/142
        self.assert_meta(
            &PairMeta::new(
                &Keys::new(&Key::new(), &Key::new(), ""),
                &old_pair,
                LINK_NAME,
                &new_pair.key(),
            )
        )

    }

    fn retract(&mut self, pair: &Pair) -> Result<(), HolochainError> {
        self.assert_meta(
            &PairMeta::new(
                &Keys::new(&Key::new(), &Key::new(), ""),
                &pair,
                STATUS_NAME,
                &CRUDStatus::DELETED.bits().to_string(),
            )
        )
    }

    fn assert_meta(&mut self, meta: &PairMeta) -> Result<(), HolochainError> {
        self.meta.insert(meta.key(), meta.clone());
        Ok(())
    }

    fn get_meta(&mut self, key: &str) -> Result<Option<PairMeta>, HolochainError> {
        Ok(self.meta.get(key).and_then(|m| Some(m.clone())))
    }

    fn get_pair_meta(&mut self, pair: &Pair) -> Result<Vec<PairMeta>, HolochainError> {
        Ok(
            self.meta
            .values()
            .filter(|&m| m.pair() == pair.key())
            .cloned()
            .collect::<Vec<PairMeta>>()
        )
    }

}

#[cfg(test)]
pub mod tests {

    use hash_table::HashTable;
    use hash_table::memory::MemTable;
    use hash_table::pair::tests::test_pair;
    use hash_table::pair::tests::test_pair_a;
    use hash_table::pair::tests::test_pair_b;

    pub fn test_table() -> MemTable {
        MemTable::new()
    }

    #[test]
    /// smoke test
    fn new() {
        test_table();
    }

    #[test]
    /// tests for ht.open()
    fn open() {
        let mut ht = test_table();
        assert_eq!(Result::Ok(()), ht.open());
    }

    #[test]
    /// tests for ht.close()
    fn close() {
        let mut ht = test_table();
        assert_eq!(Result::Ok(()), ht.close());
    }

    #[test]
    /// Pairs can round trip through table.commit() and table.get()
    fn pair_round_trip() {
        let mut ht = test_table();
        let p = test_pair();
        ht.commit(&p).unwrap();
        assert_eq!(ht.get(&p.key()), Result::Ok(Some(p)));
    }

    #[test]
    /// Pairs can be modified through table.modify()
    fn modify() {
        let mut ht = test_table();
        let p1 = test_pair_a();
        let p2 = test_pair_b();

        ht.commit(&p1).unwrap();
        ht.modify(&p1, &p2).unwrap();
    }

}
