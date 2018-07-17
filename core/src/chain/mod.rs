// pub mod memory;
use error::HolochainError;
use hash_table::HashTable;
use hash_table::{entry::Entry, pair::Pair};
// use objekt::clone_box;
use std::rc::Rc;

#[derive(Clone)]
pub struct ChainIterator {

    table: Rc<HashTable>,
    current: Option<Pair>,

}

impl ChainIterator {

    pub fn new<HT: 'static + HashTable> (table: Rc<HashTable>, pair: &Option<Pair>) -> ChainIterator {
        ChainIterator{
            current: pair.clone(),
            table: Rc::clone(&table),
        }
    }

    fn current(&self) -> Option<Pair> {
        self.current.clone()
    }

}

impl Iterator for ChainIterator {

    type Item = Pair;

    fn next(&mut self) -> Option<Pair> {
        let n = self
                .current()
                .and_then(|p| p.header().next())
                // @TODO should this panic?
                .and_then(|h| self.table.get(&h).unwrap());
        self.current = n;
        self.current()
    }

}

// #[derive(Clone, Debug, PartialEq)]
pub struct Chain<T: HashTable> {

    table: Rc<T>,
    top: Option<Pair>,

}

impl<T: HashTable> Chain<T> {

    pub fn new(table: Rc<T>) -> Chain<T> {
        Chain{
            top: None,
            table: Rc::clone(&table),
        }
    }

    pub fn top(&self) -> Option<Pair> {
        self.top.clone()
    }

    pub fn table(&self) -> Rc<T> {
        Rc::clone(&self.table)
    }

    pub fn push (&mut self, entry: &Entry) -> Result<Pair, HolochainError> {
        let pair = Pair::new(self, entry);

        if !(pair.validate()) {
            return Result::Err(HolochainError::new("attempted to push an invalid pair for this chain"))
        }

        let top_pair = self.top().and_then(|p| Some(p.key()));
        let next_pair = pair.header().next();

        if top_pair != next_pair {
            return Result::Err(HolochainError::new(
                &format!(
                    "top pair did not match next hash pair from pushed pair: {:?} vs. {:?}",
                    top_pair.clone(), next_pair.clone()
                )
            ))
        }

        // let mut validation_chain = self.clone();
        // validation_chain.top = Some(pair.clone());
        // validation_chain.pairs.insert(0, pair.clone());
        // if !validation_chain.validate() {
        //     return Result::Err(HolochainError::new("adding this pair would invalidate the source chain"))
        // }

        // @TODO implement incubator for thread safety
        // @see https://github.com/holochain/holochain-rust/issues/135
        let table = Rc::get_mut(&mut self.table).unwrap();
        let result = table.commit(&pair);
        if result.is_ok() {
            self.top = Some(pair.clone());
        }
        match result {
            Result::Ok(_) => Result::Ok(pair),
            Result::Err(e) => Result::Err(e),
        }
    }

    // fn validate(&self) -> bool {
    //     self.pairs.iter().all(|p| p.validate())
    // }
    //
    // pub fn iter(&self) -> ChainIterator {
    //     ChainIterator::new(&self.table(), &self.top())
    // }

    pub fn get (&self, k: &str) -> Result<Option<Pair>, HolochainError> {
        self.table.get(k)
    }

    // fn get_entry (&self, table: &HT, entry_hash: &str) -> Option<Pair> {
    //     // @TODO - this is a slow way to do a lookup
    //     // @see https://github.com/holochain/holochain-rust/issues/50
    //     self
    //         .iter(table)
    //         .find(|p| p.entry().hash() == entry_hash)
    // }

    pub fn top_type(&self, _t: &str) -> Option<Pair> {
        // @TODO this is wrong
        self.top()
        // self
        //     .iter()
        //     .find(|p| p.header().entry_type() == t)
    }

}

// pub trait SourceChain:
//     // IntoIterator +
//     Serialize {
//     /// append a pair to the source chain if the pair and new chain are both valid, else panic
//     fn push(&mut self, &Entry) -> Result<Pair, HolochainError>;
//
//     /// returns an iterator referencing pairs from top (most recent) to bottom (genesis)
//     fn iter(&self) -> std::slice::Iter<Pair>;
//
//     /// returns true if system and dApp validation is successful
//     fn validate(&self) -> bool;
//
//     /// returns a pair for a given header hash
//     fn get(&self, k: &str) -> Option<Pair>;
//
//     /// returns a pair for a given entry hash
//     fn get_entry(&self, k: &str) -> Option<Pair>;
//
//     /// returns the top (most recent) pair from the source chain
//     fn top(&self) -> Option<Pair>;
//
//     /// returns the top (most recent) pair of a given type from the source chain
//     fn top_type(&self, t: &str) -> Option<Pair>;
// }

#[cfg(test)]
pub mod tests {

    use super::Chain;
    use hash_table::entry::tests::test_entry;
    use hash_table::memory::tests::test_table;
    use hash_table::HashTable;
    use std::rc::Rc;
    use hash_table::memory::MemTable;

    pub fn test_chain() -> Chain<MemTable> {
        Chain::new(Rc::new(test_table()))
    }

    #[test]
    fn new() {
        test_chain();
    }

    #[test]
    fn top() {
        let c = test_chain();
        assert_eq!(None, c.top());
    }

    #[test]
    fn table() {
        let t = test_table();
        let mut c = Chain::new(Rc::new(t));
        // test that adding something to the chain adds to the table
        let p = c.push(&test_entry()).unwrap();
        let tr = Rc::new(c.table());
        assert_eq!(
            Some(p.clone()),
            c.table().get(&p.key()).unwrap(),
        );
        assert_eq!(
            Some(p.clone()),
            tr.get(&p.key()).unwrap(),
        );
        assert_eq!(
            c.table().get(&p.key()).unwrap(),
            tr.get(&p.key()).unwrap(),
        );
    }

    #[test]
    fn round_trip() {
        let mut c = test_chain();
        let e = test_entry();
        let p = c.push(&e).unwrap();
        assert_eq!(
            Some(p.clone()),
            c.get(&p.key()).unwrap(),
        );
    }

}
