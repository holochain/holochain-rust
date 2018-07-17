use std::collections::HashMap;

use error::HolochainError;

use agent::keys::Key;
use agent::keys::Keys;
use hash_table::status::StatusMask;
use hash_table::pair::Pair;
use hash_table::HashTable;
use hash_table::pair_meta::PairMeta;

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

    // fn box_clone(&self) -> Box<HashTable> {
    //     Box::new(
    //         MemTable{
    //             pairs: self.pairs.clone(),
    //             meta: self.meta.clone(),
    //         }
    //     )
    // }
    //
    // fn clone(&self) -> HashTable {
    //     MemTable{
    //         pairs: self.pairs.clone(),
    //         meta: self.meta.clone(),
    //     }
    // }

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
        self.assert_meta(
            &PairMeta::new(
                &Keys::new(&Key::new(), &Key::new(), ""),
                &old_pair,
                "status",
                &StatusMask::MODIFIED.bits().to_string(),
            )
        )
    }

    fn retract(&mut self, pair: &Pair) -> Result<(), HolochainError> {
        self.assert_meta(
            &PairMeta::new(
                &Keys::new(&Key::new(), &Key::new(), ""),
                &pair,
                "status",
                &StatusMask::DELETED.bits().to_string(),
            )
        )
    }

    // EAVTK
    // pair, attribute name, attribute value, txn id, source, signature
    fn assert_meta(&mut self, meta: &PairMeta) -> Result<(), HolochainError> {
        self.meta.insert(meta.key(), meta.clone());
        Ok(())
    }

}

#[cfg(test)]
pub mod tests {

    use hash_table::HashTable;
    use hash_table::memory::MemTable;
    use hash_table::pair::tests::test_pair;

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
    /// round trip
    fn round_trip() {
        let mut ht = test_table();
        let p = test_pair();
        ht.commit(&p).unwrap();
        assert_eq!(ht.get(&p.key()), Result::Ok(Some(p)));
    }

}
