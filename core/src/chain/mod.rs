// pub mod memory;
use error::HolochainError;
use hash_table::HashTable;
use hash_table::{entry::Entry, pair::Pair};
use std::rc::Rc;

#[derive(Clone)]
pub struct ChainIterator<T: HashTable> {

    table: Rc<T>,
    current: Option<Pair>,

}

impl<T: HashTable> ChainIterator<T> {

    pub fn new (table: Rc<T>, pair: &Option<Pair>) -> ChainIterator<T> {
        ChainIterator{
            current: pair.clone(),
            table: Rc::clone(&table),
        }
    }

    fn current(&self) -> Option<Pair> {
        self.current.clone()
    }

}

impl<T: HashTable> Iterator for ChainIterator<T> {

    type Item = Pair;

    fn next(&mut self) -> Option<Pair> {
        let ret = self.current();
        self.current = ret.clone()
                        .and_then(|p| p.header().next())
                        // @TODO should this panic?
                        .and_then(|h| self.table.get(&h).unwrap());
        ret
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

    pub fn validate(&self) -> bool {
        self.iter().all(|p| p.validate())
    }

    pub fn iter(&self) -> ChainIterator<T> {
        ChainIterator::new(self.table(), &self.top())
    }

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

#[cfg(test)]
pub mod tests {

    use super::Chain;
    use hash_table::entry::tests::test_entry;
    use hash_table::entry::tests::test_entry_a;
    use hash_table::entry::tests::test_entry_b;
    use hash_table::memory::tests::test_table;
    use hash_table::HashTable;
    use hash_table::pair::Pair;
    use std::rc::Rc;
    use hash_table::memory::MemTable;

    /// builds a dummy chain for testing
    pub fn test_chain() -> Chain<MemTable> {
        Chain::new(Rc::new(test_table()))
    }

    #[test]
    /// smoke test for new chains
    fn new() {
        test_chain();
    }

    #[test]
    /// tests for chain.top()
    fn top() {
        let mut chain = test_chain();
        assert_eq!(None, chain.top());

        let e1 = test_entry_a();
        let e2 = test_entry_b();

        let p1 = chain.push(&e1).unwrap();
        assert_eq!(Some(p1), chain.top());

        let p2 = chain.push(&e2).unwrap();
        assert_eq!(Some(p2), chain.top());
    }

    #[test]
    /// tests for chain.table()
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
    /// tests for chain.push()
    fn push() {
        let mut chain = test_chain();

        assert_eq!(None, chain.top());

        // chain top, pair entry and headers should all line up after a push
        let e1 = test_entry_a();
        let p1 = chain.push(&e1).unwrap();

        assert_eq!(Some(p1.clone()), chain.top());
        assert_eq!(e1, p1.entry());
        assert_eq!(e1.hash(), p1.header().entry());

        // we should be able to do it again
        let e2 = test_entry_b();
        let p2 = chain.push(&e2).unwrap();

        assert_eq!(Some(p2.clone()), chain.top());
        assert_eq!(e2, p2.entry());
        assert_eq!(e2.hash(), p2.header().entry());
    }

    #[test]
    /// test chain.push() and chain.get() together
    fn round_trip() {
        let mut c = test_chain();
        let e = test_entry();
        let p = c.push(&e).unwrap();
        assert_eq!(
            Some(p.clone()),
            c.get(&p.key()).unwrap(),
        );
    }

    #[test]
    /// test chain.validate()
    fn validate() {
        let mut chain = test_chain();

        let e1 = test_entry_a();
        let e2 = test_entry_b();

        assert!(chain.validate());

        chain.push(&e1).unwrap();
        assert!(chain.validate());

        chain.push(&e2).unwrap();
        assert!(chain.validate());
    }

    #[test]
    /// test chain.iter()
    fn iter() {
        let mut chain = test_chain();

        let e1 = test_entry_a();
        let e2 = test_entry_b();

        let p1 = chain.push(&e1).unwrap();
        let p2 = chain.push(&e2).unwrap();

        assert_eq!(vec![p2, p1], chain.iter().collect::<Vec<Pair>>());
    }

    #[test]
    /// test chain.iter() functional interface
    fn iter_functional() {
        let mut chain = test_chain();

        let e1 = test_entry_a();
        let e2 = test_entry_b();

        let p1 = chain.push(&e1).unwrap();
        let _p2 = chain.push(&e2).unwrap();
        let p3 = chain.push(&e1).unwrap();

        assert_eq!(
            vec![p3, p1],
            chain
                .iter()
                .filter(|p| p.entry().entry_type() == "testEntryType")
                .collect::<Vec<Pair>>()
        );
    }

    #[test]
    /// test chain.get()
    fn get() {
        let mut chain = test_chain();

        let e1 = test_entry_a();
        let e2 = test_entry_b();
        let e3 = test_entry_a();

        let p1 = chain.push(&e1).unwrap();
        let p2 = chain.push(&e2).unwrap();
        let p3 = chain.push(&e3).unwrap();

        assert_eq!(None, chain.get("").unwrap());
        assert_eq!(Some(p1.clone()), chain.get(&p1.key()).unwrap());
        assert_eq!(Some(p2.clone()), chain.get(&p2.key()).unwrap());
        assert_eq!(Some(p3.clone()), chain.get(&p3.key()).unwrap());
    }

}
