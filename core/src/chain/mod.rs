// pub mod memory;
use error::HolochainError;
use hash_table::{entry::Entry, pair::Pair, HashTable};
use serde_json;
use std::{fmt, rc::Rc};

#[derive(Clone)]
pub struct ChainIterator<T: HashTable> {
    // @TODO thread safe table references
    // @see https://github.com/holochain/holochain-rust/issues/135
    table: Rc<T>,
    current: Option<Pair>,
}

impl<T: HashTable> ChainIterator<T> {
    pub fn new(table: Rc<T>, pair: &Option<Pair>) -> ChainIterator<T> {
        ChainIterator {
            current: pair.clone(),
            table: Rc::clone(&table),
        }
    }

    /// returns the current pair representing the iterator internal state
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
                        // @see https://github.com/holochain/holochain-rust/issues/146
                        .and_then(|h| self.table.get(&h).unwrap());
        ret
    }
}

pub struct Chain<T: HashTable> {
    // @TODO thread safe table references
    // @see https://github.com/holochain/holochain-rust/issues/135
    table: Rc<T>,
    top: Option<Pair>,
}

impl<T: HashTable> PartialEq for Chain<T> {
    fn eq(&self, other: &Chain<T>) -> bool {
        // an invalid chain is like NaN... not even equal to itself
        self.validate() &&
        other.validate() &&
        // header hashing ensures that if the tops match the whole chain matches
        self.top() == other.top()
    }
}

impl<T: HashTable> Eq for Chain<T> {}

impl<T: HashTable> fmt::Debug for Chain<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Chain {{ top: {:?} }}", self.top)
    }
}

impl<T: HashTable> IntoIterator for Chain<T> {
    type Item = Pair;
    type IntoIter = ChainIterator<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T: HashTable> Chain<T> {
    /// build a new Chain against an existing HashTable
    pub fn new(table: Rc<T>) -> Chain<T> {
        Chain {
            top: None,
            table: Rc::clone(&table),
        }
    }

    /// returns a clone of the top Pair
    pub fn top(&self) -> Option<Pair> {
        self.top.clone()
    }

    /// returns a reference to the underlying HashTable
    pub fn table(&self) -> Rc<T> {
        Rc::clone(&self.table)
    }

    /// private pair-oriented version of push() (which expects Entries)
    fn push_pair(&mut self, pair: Pair) -> Result<Pair, HolochainError> {
        if !(pair.validate()) {
            return Err(HolochainError::new(
                "attempted to push an invalid pair for this chain",
            ));
        }

        let top_pair = self.top().and_then(|p| Some(p.key()));
        let next_pair = pair.header().next();

        if top_pair != next_pair {
            return Err(HolochainError::new(&format!(
                "top pair did not match next hash pair from pushed pair: {:?} vs. {:?}",
                top_pair.clone(),
                next_pair.clone()
            )));
        }

        // @TODO implement incubator for thread safety
        // @see https://github.com/holochain/holochain-rust/issues/135
        let table = Rc::get_mut(&mut self.table).unwrap();
        let result = table.commit(&pair);
        if result.is_ok() {
            self.top = Some(pair.clone());
        }
        match result {
            Ok(_) => Ok(pair),
            Err(e) => Err(e),
        }
    }

    /// push a new Entry on to the top of the Chain
    /// the Pair for the new Entry is automatically generated and validated against the current top
    /// Pair to ensure the chain links up correctly across the underlying table data
    /// the newly created and pushed Pair is returned in the fn Result
    pub fn push(&mut self, entry: &Entry) -> Result<Pair, HolochainError> {
        let pair = Pair::new(self, entry);
        self.push_pair(pair)
    }

    /// returns true if all pairs in the chain pass validation
    pub fn validate(&self) -> bool {
        self.iter().all(|p| p.validate())
    }

    /// returns a ChainIterator that provides cloned Pairs from the underlying HashTable
    pub fn iter(&self) -> ChainIterator<T> {
        ChainIterator::new(self.table(), &self.top())
    }

    /// get a Pair by Pair/Header key from the HashTable if it exists
    pub fn get(&self, k: &str) -> Result<Option<Pair>, HolochainError> {
        self.table.get(k)
    }

    /// get an Entry by Entry key from the HashTable if it exists
    pub fn get_entry(&self, entry_hash: &str) -> Result<Option<Pair>, HolochainError> {
        // @TODO - this is a slow way to do a lookup
        // @see https://github.com/holochain/holochain-rust/issues/50
        Ok(self
                .iter()
                // @TODO entry hashes are NOT unique across pairs so k/v lookups can't be 1:1
                // @see https://github.com/holochain/holochain-rust/issues/145
                .find(|p| p.entry().hash() == entry_hash))
    }

    /// get the top Pair by Entry type
    pub fn top_type(&self, t: &str) -> Result<Option<Pair>, HolochainError> {
        Ok(self.iter().find(|p| p.header().entry_type() == t))
    }

    /// get the entire chain, top to bottom as a JSON array
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        let as_seq = self.iter().collect::<Vec<Pair>>();
        serde_json::to_string(&as_seq)
    }

    /// restore a valid JSON chain
    pub fn from_json(table: Rc<T>, s: &str) -> Self {
        // @TODO inappropriate unwrap?
        let mut as_seq: Vec<Pair> = serde_json::from_str(s).unwrap();
        as_seq.reverse();

        let mut chain = Chain::new(table);
        for p in as_seq {
            chain.push_pair(p).unwrap();
        }
        chain
    }
}

#[cfg(test)]
pub mod tests {

    use super::Chain;
    use hash_table::{
        entry::tests::{test_entry, test_entry_a, test_entry_b, test_type_a, test_type_b},
        memory::{tests::test_table, MemTable}, pair::Pair, HashTable,
    };
    use std::rc::Rc;

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
    /// test chain equality
    fn eq() {
        let mut c1 = test_chain();
        let mut c2 = test_chain();
        let mut c3 = test_chain();

        let e1 = test_entry_a();
        let e2 = test_entry_b();

        c1.push(&e1).unwrap();
        c2.push(&e1).unwrap();
        c3.push(&e2).unwrap();

        assert_eq!(c1.top(), c2.top());
        assert_eq!(c1, c2);

        assert_ne!(c1, c3);
        assert_ne!(c2, c3);
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
        assert_eq!(Some(p.clone()), c.table().get(&p.key()).unwrap(),);
        assert_eq!(Some(p.clone()), tr.get(&p.key()).unwrap(),);
        assert_eq!(c.table().get(&p.key()).unwrap(), tr.get(&p.key()).unwrap(),);
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
    /// test chain.push() and chain.get() together
    fn round_trip() {
        let mut c = test_chain();
        let e = test_entry();
        let p = c.push(&e).unwrap();
        assert_eq!(Some(p.clone()), c.get(&p.key()).unwrap(),);
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
        assert_eq!(Some(p1.clone()), chain.get(&p1.header().key()).unwrap());
        assert_eq!(Some(p2.clone()), chain.get(&p2.header().key()).unwrap());
        assert_eq!(Some(p3.clone()), chain.get(&p3.header().key()).unwrap());
    }

    #[test]
    /// test chain.get_entry()
    fn get_entry() {
        let mut chain = test_chain();

        let e1 = test_entry_a();
        let e2 = test_entry_b();
        let e3 = test_entry_a();

        let p1 = chain.push(&e1).unwrap();
        let p2 = chain.push(&e2).unwrap();
        let p3 = chain.push(&e3).unwrap();

        assert_eq!(None, chain.get_entry("").unwrap());
        // @TODO at this point we have p3 with the same entry key as p1...
        assert_eq!(
            Some(p3.clone()),
            chain.get_entry(&p1.entry().key()).unwrap()
        );
        assert_eq!(
            Some(p2.clone()),
            chain.get_entry(&p2.entry().key()).unwrap()
        );
        assert_eq!(
            Some(p3.clone()),
            chain.get_entry(&p3.entry().key()).unwrap()
        );
    }

    #[test]
    /// test chain.top_type()
    fn top_type() {
        let mut chain = test_chain();

        assert_eq!(None, chain.top_type(&test_type_a()).unwrap());
        assert_eq!(None, chain.top_type(&test_type_b()).unwrap());

        let e1 = test_entry_a();
        let e2 = test_entry_b();
        let e3 = test_entry_a();

        // type a should be p1
        // type b should be None
        let p1 = chain.push(&e1).unwrap();
        assert_eq!(Some(p1.clone()), chain.top_type(&test_type_a()).unwrap());
        assert_eq!(None, chain.top_type(&test_type_b()).unwrap());

        // type a should still be p1
        // type b should be p2
        let p2 = chain.push(&e2).unwrap();
        assert_eq!(Some(p1.clone()), chain.top_type(&test_type_a()).unwrap());
        assert_eq!(Some(p2.clone()), chain.top_type(&test_type_b()).unwrap());

        // type a should be p3
        // type b should still be p2
        let p3 = chain.push(&e3).unwrap();
        assert_eq!(Some(p3.clone()), chain.top_type(&test_type_a()).unwrap());
        assert_eq!(Some(p2.clone()), chain.top_type(&test_type_b()).unwrap());
    }

    #[test]
    /// test IntoIterator implementation
    fn into_iter() {
        let mut chain = test_chain();

        let e1 = test_entry_a();
        let e2 = test_entry_b();
        let e3 = test_entry_a();

        let p1 = chain.push(&e1).unwrap();
        let p2 = chain.push(&e2).unwrap();
        let p3 = chain.push(&e3).unwrap();

        // into_iter() returns clones of pairs
        let mut i = 0;
        let expected = [p3.clone(), p2.clone(), p1.clone()];
        for p in chain {
            assert_eq!(expected[i], p);
            i = i + 1;
        }
    }

    #[test]
    /// test to_json() and from_json() implementation
    fn json_round_trip() {
        let mut chain = test_chain();

        let e1 = test_entry_a();
        let e2 = test_entry_b();
        let e3 = test_entry_a();

        chain.push(&e1).unwrap();
        chain.push(&e2).unwrap();
        chain.push(&e3).unwrap();

        let expected_json = "[{\"header\":{\"entry_type\":\"testEntryType\",\"time\":\"\",\"next\":\"QmPT5HXvyv54Dg36YSK1A2rYvoPCNWoqpLzzZnHnQBcU6x\",\"entry\":\"QmbXSE38SN3SuJDmHKSSw5qWWegvU7oTxrLDRavWjyxMrT\",\"type_next\":\"QmawqBCVVap9KdaakqEHF4JzUjjLhmR7DpM5jgJko8j1rA\",\"signature\":\"\"},\"entry\":{\"content\":\"test entry content\",\"entry_type\":\"testEntryType\"}},{\"header\":{\"entry_type\":\"testEntryTypeB\",\"time\":\"\",\"next\":\"QmawqBCVVap9KdaakqEHF4JzUjjLhmR7DpM5jgJko8j1rA\",\"entry\":\"QmPz5jKXsxq7gPVAbPwx5gD2TqHfqB8n25feX5YH18JXrT\",\"type_next\":null,\"signature\":\"\"},\"entry\":{\"content\":\"other test entry content\",\"entry_type\":\"testEntryTypeB\"}},{\"header\":{\"entry_type\":\"testEntryType\",\"time\":\"\",\"next\":null,\"entry\":\"QmbXSE38SN3SuJDmHKSSw5qWWegvU7oTxrLDRavWjyxMrT\",\"type_next\":null,\"signature\":\"\"},\"entry\":{\"content\":\"test entry content\",\"entry_type\":\"testEntryType\"}}]";
        assert_eq!(expected_json, chain.to_json().unwrap());

        let table = test_table();
        assert_eq!(chain, Chain::from_json(Rc::new(table), expected_json));
    }

}
