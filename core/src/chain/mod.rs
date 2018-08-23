// pub mod memory;
use error::HolochainError;
use hash_table::{entry::Entry, pair::Pair, HashTable};
use serde_json;
use std::{fmt, rc::Rc};

/// Iterator type for pairs in a chain
/// next method may panic if there is an error in the underlying table
#[derive(Clone)]
pub struct ChainIterator<T: HashTable> {
    // @TODO thread safe table references
    // @see https://github.com/holochain/holochain-rust/issues/135
    table: Rc<T>,
    current: Option<Pair>,
}

impl<T: HashTable> ChainIterator<T> {
    // @TODO table implementation is changing anyway so waste of time to mess with ref/value
    // @see https://github.com/holochain/holochain-rust/issues/135
    #[allow(unknown_lints)]
    #[allow(needless_pass_by_value)]
    pub fn new(table: Rc<T>, pair: Option<Pair>) -> ChainIterator<T> {
        ChainIterator {
            current: pair,
            table: Rc::clone(&table),
        }
    }
}

impl<T: HashTable> Iterator for ChainIterator<T> {
    type Item = Pair;

    /// May panic if there is an underlying error in the table
    fn next(&mut self) -> Option<Pair> {
        let previous = self.current.take();
        self.current = previous.as_ref()
                        .and_then(|p| p.header().prev())
                        // @TODO should this panic?
                        // @see https://github.com/holochain/holochain-rust/issues/146
                        .and_then(|h| self.table.get(&h).expect("getting from a table shouldn't fail"));
        previous
    }
}

/// Struct representing the source chain.
/// It mostly just manages the HashTable and adds extra logic
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

/// Turns a chain into an iterator over it's Pairs
impl<T: HashTable> IntoIterator for Chain<T> {
    type Item = Pair;
    type IntoIter = ChainIterator<T>;

    /// returns a ChainIterator that provides cloned Pairs from the underlying HashTable
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T: HashTable> Chain<T> {
    // @TODO table implementation is changing anyway so waste of time to mess with ref/value
    // @see https://github.com/holochain/holochain-rust/issues/135
    #[allow(unknown_lints)]
    #[allow(needless_pass_by_value)]
    /// build a new Chain against an existing HashTable
    pub fn new(table: Rc<T>) -> Chain<T> {
        Chain {
            top: None,
            table: Rc::clone(&table),
        }
    }

    /// returns a reference to the top Pair
    pub fn top(&self) -> &Option<Pair> {
        &self.top
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

        let top_pair = self.top().as_ref().map(|p| p.key());
        let next_pair = pair.header().prev();

        if top_pair != next_pair {
            return Err(HolochainError::new(&format!(
                "top pair did not match next hash pair from pushed pair: {:?} vs. {:?}",
                top_pair, next_pair,
            )));
        }

        // @TODO implement incubator for thread safety
        // @see https://github.com/holochain/holochain-rust/issues/135
        let table = Rc::get_mut(&mut self.table).ok_or(HolochainError::new(
            "attempted to push while table is already borrowed",
        ))?;
        table.commit(&pair)?;
        self.top = Some(pair.clone());
        Ok(pair)
    }

    /// push a new Entry on to the top of the Chain
    /// the Pair for the new Entry is automatically generated and validated against the current top
    /// Pair to ensure the chain links up correctly across the underlying table data
    /// the newly created and pushed Pair is returned in the fn Result
    pub fn push_entry(&mut self, entry: &Entry) -> Result<Pair, HolochainError> {
        let pair = Pair::new(self, entry.clone());
        self.push_pair(pair)
    }

    /// returns true if all pairs in the chain pass validation
    pub fn validate(&self) -> bool {
        self.iter().all(|p| p.validate())
    }

    /// returns a ChainIterator that provides cloned Pairs from the underlying HashTable
    pub fn iter(&self) -> ChainIterator<T> {
        ChainIterator::new(self.table(), self.top().clone())
    }

    /// get a Pair by Pair/Header key from the HashTable if it exists
    pub fn get_pair(&self, k: &str) -> Result<Option<Pair>, HolochainError> {
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

    /// get the entire chain, top to bottom as a JSON array or canonical pairs
    /// @TODO return canonical JSON
    /// @see https://github.com/holochain/holochain-rust/issues/75
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        let as_seq = self.iter().collect::<Vec<Pair>>();
        serde_json::to_string(&as_seq)
    }

    /// restore canonical JSON chain
    ///
    /// # Panics
    ///
    /// Panics if the string passed isn't valid JSON or pairs fail to validate
    ///
    /// @TODO accept canonical JSON
    /// @see https://github.com/holochain/holochain-rust/issues/75
    pub fn from_json(table: Rc<T>, s: &str) -> Self {
        // @TODO inappropriate expect?
        // @see https://github.com/holochain/holochain-rust/issues/168
        let mut as_seq: Vec<Pair> = serde_json::from_str(s).expect("argument should be valid json");
        as_seq.reverse();

        let mut chain = Chain::new(table);
        for p in as_seq {
            chain.push_pair(p).expect("pair should be valid");
        }
        chain
    }
}

#[cfg(test)]
pub mod tests {

    use super::Chain;
    use hash_table::{
        entry::tests::{test_entry, test_entry_a, test_entry_b, test_type_a, test_type_b},
        memory::{tests::test_table, MemTable},
        pair::Pair,
        HashTable,
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

        c1.push_entry(&e1)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");
        c2.push_entry(&e1)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");
        c3.push_entry(&e2)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");

        assert_eq!(c1.top(), c2.top());
        assert_eq!(c1, c2);

        assert_ne!(c1, c3);
        assert_ne!(c2, c3);
    }

    #[test]
    /// tests for chain.top()
    fn top() {
        let mut chain = test_chain();
        assert_eq!(&None, chain.top());

        let e1 = test_entry_a();
        let e2 = test_entry_b();

        let p1 = chain
            .push_entry(&e1)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");
        assert_eq!(&Some(p1), chain.top());

        let p2 = chain
            .push_entry(&e2)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");
        assert_eq!(&Some(p2), chain.top());
    }

    #[test]
    /// tests for chain.table()
    fn table() {
        let t = test_table();
        let mut c = Chain::new(Rc::new(t));
        // test that adding something to the chain adds to the table
        let p = c
            .push_entry(&test_entry())
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");
        let tr = Rc::new(c.table());
        let chain_entry = c
            .table()
            .get(&p.key())
            .expect("getting an entry from a chain shouldn't fail");
        assert_eq!(Some(&p), chain_entry.as_ref());
        let tr_entry = tr
            .get(&p.key())
            .expect("getting an entry from a chain shouldn't fail");
        assert_eq!(Some(&p), tr_entry.as_ref());
        assert_eq!(chain_entry, tr_entry);
    }

    #[test]
    /// tests for chain.push()
    fn push() {
        let mut chain = test_chain();

        assert_eq!(&None, chain.top());

        // chain top, pair entry and headers should all line up after a push
        let e1 = test_entry_a();
        let p1 = chain
            .push_entry(&e1)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");

        assert_eq!(Some(&p1), chain.top().as_ref());
        assert_eq!(&e1, p1.entry());
        assert_eq!(e1.hash(), p1.header().entry_hash());

        // we should be able to do it again
        let e2 = test_entry_b();
        let p2 = chain
            .push_entry(&e2)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");

        assert_eq!(Some(&p2), chain.top().as_ref());
        assert_eq!(&e2, p2.entry());
        assert_eq!(e2.hash(), p2.header().entry_hash());
    }

    #[test]
    /// test chain.validate()
    fn validate() {
        let mut chain = test_chain();

        let e1 = test_entry_a();
        let e2 = test_entry_b();

        assert!(chain.validate());

        chain
            .push_entry(&e1)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");
        assert!(chain.validate());

        chain
            .push_entry(&e2)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");
        assert!(chain.validate());
    }

    #[test]
    /// test chain.push() and chain.get() together
    fn round_trip() {
        let mut c = test_chain();
        let e = test_entry();
        let p = c
            .push_entry(&e)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");
        assert_eq!(
            Some(&p),
            c.get_pair(&p.key())
                .expect("getting an entry from a chain shouldn't fail")
                .as_ref()
        );
    }

    #[test]
    /// test chain.iter()
    fn iter() {
        let mut chain = test_chain();

        let e1 = test_entry_a();
        let e2 = test_entry_b();

        let p1 = chain
            .push_entry(&e1)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");
        let p2 = chain
            .push_entry(&e2)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");

        assert_eq!(vec![p2, p1], chain.iter().collect::<Vec<Pair>>());
    }

    #[test]
    /// test chain.iter() functional interface
    fn iter_functional() {
        let mut chain = test_chain();

        let e1 = test_entry_a();
        let e2 = test_entry_b();

        let p1 = chain
            .push_entry(&e1)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");
        let _p2 = chain
            .push_entry(&e2)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");
        let p3 = chain
            .push_entry(&e1)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");

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

        let p1 = chain
            .push_entry(&e1)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");
        let p2 = chain
            .push_entry(&e2)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");
        let p3 = chain
            .push_entry(&e3)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");

        assert_eq!(
            None,
            chain
                .get_pair("")
                .expect("getting an entry from a chain shouldn't fail")
        );
        assert_eq!(
            Some(&p1),
            chain
                .get_pair(&p1.key())
                .expect("getting an entry from a chain shouldn't fail")
                .as_ref()
        );
        assert_eq!(
            Some(&p2),
            chain
                .get_pair(&p2.key())
                .expect("getting an entry from a chain shouldn't fail")
                .as_ref()
        );
        assert_eq!(
            Some(&p3),
            chain
                .get_pair(&p3.key())
                .expect("getting an entry from a chain shouldn't fail")
                .as_ref()
        );

        assert_eq!(
            Some(&p1),
            chain
                .get_pair(&p1.header().key())
                .expect("getting an entry from a chain shouldn't fail")
                .as_ref()
        );
        assert_eq!(
            Some(&p2),
            chain
                .get_pair(&p2.header().key())
                .expect("getting an entry from a chain shouldn't fail")
                .as_ref()
        );
        assert_eq!(
            Some(&p3),
            chain
                .get_pair(&p3.header().key())
                .expect("getting an entry from a chain shouldn't fail")
                .as_ref()
        );
    }

    #[test]
    /// test chain.get_entry()
    fn get_entry() {
        let mut chain = test_chain();

        let e1 = test_entry_a();
        let e2 = test_entry_b();
        let e3 = test_entry_a();

        let p1 = chain
            .push_entry(&e1)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");
        let p2 = chain
            .push_entry(&e2)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");
        let p3 = chain
            .push_entry(&e3)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");

        assert_eq!(
            None,
            chain
                .get_entry("")
                .expect("getting an entry from a chain shouldn't fail")
        );
        // @TODO at this point we have p3 with the same entry key as p1...
        assert_eq!(
            Some(&p3),
            chain
                .get_entry(&p1.entry().key())
                .expect("getting an entry from a chain shouldn't fail")
                .as_ref()
        );
        assert_eq!(
            Some(&p2),
            chain
                .get_entry(&p2.entry().key())
                .expect("getting an entry from a chain shouldn't fail")
                .as_ref()
        );
        assert_eq!(
            Some(&p3),
            chain
                .get_entry(&p3.entry().key())
                .expect("getting an entry from a chain shouldn't fail")
                .as_ref()
        );
    }

    #[test]
    /// test chain.top_type()
    fn top_type() {
        let mut chain = test_chain();

        assert_eq!(
            None,
            chain
                .top_type(&test_type_a())
                .expect("finding top entry of a given type shouldn't fail")
        );
        assert_eq!(
            None,
            chain
                .top_type(&test_type_b())
                .expect("finding top entry of a given type shouldn't fail")
        );

        let e1 = test_entry_a();
        let e2 = test_entry_b();
        let e3 = test_entry_a();

        // type a should be p1
        // type b should be None
        let p1 = chain
            .push_entry(&e1)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");
        assert_eq!(
            Some(&p1),
            chain
                .top_type(&test_type_a())
                .expect("finding top entry of a given type shouldn't fail")
                .as_ref()
        );
        assert_eq!(
            None,
            chain
                .top_type(&test_type_b())
                .expect("finding top entry of a given type shouldn't fail")
        );

        // type a should still be p1
        // type b should be p2
        let p2 = chain
            .push_entry(&e2)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");
        assert_eq!(
            Some(&p1),
            chain
                .top_type(&test_type_a())
                .expect("finding top entry of a given type shouldn't fail")
                .as_ref()
        );
        assert_eq!(
            Some(&p2),
            chain
                .top_type(&test_type_b())
                .expect("finding top entry of a given type shouldn't fail")
                .as_ref()
        );

        // type a should be p3
        // type b should still be p2
        let p3 = chain
            .push_entry(&e3)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");

        assert_eq!(
            Some(&p3),
            chain
                .top_type(&test_type_a())
                .expect("finding top entry of a given type shouldn't fail")
                .as_ref()
        );
        assert_eq!(
            Some(&p2),
            chain
                .top_type(&test_type_b())
                .expect("finding top entry of a given type shouldn't fail")
                .as_ref()
        );
    }

    #[test]
    /// test IntoIterator implementation
    fn into_iter() {
        let mut chain = test_chain();

        let e1 = test_entry_a();
        let e2 = test_entry_b();
        let e3 = test_entry_a();

        let p1 = chain
            .push_entry(&e1)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");
        let p2 = chain
            .push_entry(&e2)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");
        let p3 = chain
            .push_entry(&e3)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");

        // into_iter() returns clones of pairs
        assert_eq!(vec![p3, p2, p1], chain.into_iter().collect::<Vec<Pair>>());
    }

    #[test]
    /// test to_json() and from_json() implementation
    fn json_round_trip() {
        let mut chain = test_chain();

        let e1 = test_entry_a();
        let e2 = test_entry_b();
        let e3 = test_entry_a();

        chain
            .push_entry(&e1)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");
        chain
            .push_entry(&e2)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");
        chain
            .push_entry(&e3)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");

        let expected_json = "[{\"header\":{\"entry_type\":\"testEntryType\",\"timestamp\":\"\",\"prev\":\"QmPT5HXvyv54Dg36YSK1A2rYvoPCNWoqpLzzZnHnQBcU6x\",\"entry_hash\":\"QmbXSE38SN3SuJDmHKSSw5qWWegvU7oTxrLDRavWjyxMrT\",\"entry_signature\":\"\",\"prev_same\":\"QmawqBCVVap9KdaakqEHF4JzUjjLhmR7DpM5jgJko8j1rA\"},\"entry\":{\"content\":\"test entry content\",\"entry_type\":\"testEntryType\"}},{\"header\":{\"entry_type\":\"testEntryTypeB\",\"timestamp\":\"\",\"prev\":\"QmawqBCVVap9KdaakqEHF4JzUjjLhmR7DpM5jgJko8j1rA\",\"entry_hash\":\"QmPz5jKXsxq7gPVAbPwx5gD2TqHfqB8n25feX5YH18JXrT\",\"entry_signature\":\"\",\"prev_same\":null},\"entry\":{\"content\":\"other test entry content\",\"entry_type\":\"testEntryTypeB\"}},{\"header\":{\"entry_type\":\"testEntryType\",\"timestamp\":\"\",\"prev\":null,\"entry_hash\":\"QmbXSE38SN3SuJDmHKSSw5qWWegvU7oTxrLDRavWjyxMrT\",\"entry_signature\":\"\",\"prev_same\":null},\"entry\":{\"content\":\"test entry content\",\"entry_type\":\"testEntryType\"}}]"
        ;
        assert_eq!(
            expected_json,
            chain.to_json().expect("chain shouldn't fail to serialize")
        );

        let table = test_table();
        assert_eq!(chain, Chain::from_json(Rc::new(table), expected_json));
    }

}
