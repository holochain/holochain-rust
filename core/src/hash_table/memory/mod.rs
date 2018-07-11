use std::collections::HashMap;

use error::HolochainError;

use hash_table::status::StatusMask;
use hash_table::pair::Pair;
use hash_table::HashTable;

#[derive(Serialize, Debug, Clone)]
pub struct MemTable {
    pairs: HashMap<String, Pair>,
    meta: HashMap<String, String>,
}

impl MemTable {

    pub fn new() -> MemTable {

        MemTable{
            pairs: HashMap::new(),
        }

    }

}

impl HashTable for MemTable {

    fn box_clone(&self) -> Box<HashTable> {
        Box::new(
            MemTable{
                pairs: self.pairs.clone(),
            }
        )
    }

    fn open(&mut self) -> Result<(), HolochainError> {
        Result::Ok(())
    }

    fn close(&mut self) -> Result<(), HolochainError> {
        Result::Ok(())
    }

    fn commit(&mut self, pair: &Pair) -> Result<(), HolochainError> {
        self.pairs.insert(pair.hash(), pair.clone());
        Result::Ok(())
    }

    fn get(&self, key: &str) -> Result<Option<Pair>, HolochainError> {
        Result::Ok(self.pairs.get(key.into()).and_then(|p| Some(p.clone())))
    }

    fn modify(&mut self, old_pair: &Pair, new_pair: &Pair) -> Result<(), HolochainError> {
        self.commit(new_pair);
        // @TODO better meta
        self.assert_meta(
            old_pair,
            "status",
            &StatusMask::MODIFIED.bits().to_string(),
        );
        Result::Ok(())
    }

    fn retract(&mut self, pair: &Pair) -> Result<(), HolochainError> {
        Result::Ok(())
    }

    // EAVTK
    // pair, attribute name, attribute value, txn id, source, signature
    fn assert_meta(&mut self, e: &Pair, a: &str, v: &str, t: u32, s: &str, sig: &str) -> Result<(), HolochainError> {
        Result::Ok(())
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
        ht.commit(&p);
        assert_eq!(ht.get(&p.hash()), Result::Ok(Some(p)));
    }

}
