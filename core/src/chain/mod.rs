use actor::Protocol;
use error::HolochainError;
use hash_table::{actor::AskHashTable, entry::Entry, pair::Pair, HashTable};
use riker::actors::*;
use serde_json;
use std::fmt;
// use futures::executor::block_on;

#[derive(Clone)]
pub struct ChainIterator {
    table: ActorRef<Protocol>,
    current: Option<Pair>,
}

impl ChainIterator {
    // @TODO table implementation is changing anyway so waste of time to mess with ref/value
    // @see https://github.com/holochain/holochain-rust/issues/135
    #[allow(unknown_lints)]
    #[allow(needless_pass_by_value)]
    pub fn new(table: ActorRef<Protocol>, pair: &Option<Pair>) -> ChainIterator {
        ChainIterator {
            current: pair.clone(),
            table: table.clone(),
        }
    }

    /// returns the current pair representing the iterator internal state
    fn current(&self) -> Option<Pair> {
        self.current.clone()
    }
}

impl Iterator for ChainIterator {
    type Item = Pair;

    fn next(&mut self) -> Option<Pair> {
        let ret = self.current();
        println!("next current: {:?}", ret);
        self.current = ret.clone()
                        .and_then(|p| p.header().next())
                        // @TODO should this panic?
                        // @see https://github.com/holochain/holochain-rust/issues/146
                        .and_then(|h| {
                            // let response = self.table.ask(Protocol::HashTableGetPair(h.to_string()));
                            // let result = unwrap_to!(response => Protocol::HashTableGetPairResult);
                            // println!("next: {:?}", result);
                            // result.clone().unwrap()
                            self.table.get(&h.to_string()).unwrap()
                        });
        ret
    }
}

#[derive(Clone)]
pub struct Chain {
    // @TODO thread safe table references
    // @see https://github.com/holochain/holochain-rust/issues/135
    table: ActorRef<Protocol>,
    top_pair: Option<Pair>,
}

impl PartialEq for Chain {
    fn eq(&self, other: &Chain) -> bool {
        // an invalid chain is like NaN... not even equal to itself
        self.validate() &&
        other.validate() &&
        // header hashing ensures that if the tops match the whole chain matches
        self.top_pair() == other.top_pair()
    }
}

impl Eq for Chain {}

impl fmt::Debug for Chain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Chain {{ top: {:?} }}", self.top_pair)
    }
}

impl IntoIterator for Chain {
    type Item = Pair;
    type IntoIter = ChainIterator;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl Chain {
    pub fn new(table: ActorRef<Protocol>) -> Chain {
        Chain {
            top_pair: None,
            table: table.clone(),
        }
    }

    /// returns a reference to the underlying HashTable
    pub fn table(&self) -> ActorRef<Protocol> {
        self.table.clone()
    }

    /// returns true if all pairs in the chain pass validation
    fn validate(&self) -> bool {
        self.iter().all(|p| p.validate())
    }

    /// returns a ChainIterator that provides cloned Pairs from the underlying HashTable
    fn iter(&self) -> ChainIterator {
        println!("at iter: {:?}", self);
        ChainIterator::new(self.table(), &self.top_pair())
    }

    /// get the entire chain, top to bottom as a JSON array or canonical pairs
    /// @TODO return canonical JSON
    /// @see https://github.com/holochain/holochain-rust/issues/75
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        let as_seq = self.iter().collect::<Vec<Pair>>();
        serde_json::to_string(&as_seq)
    }

    /// restore canonical JSON chain
    /// @TODO accept canonical JSON
    /// @see https://github.com/holochain/holochain-rust/issues/75
    pub fn from_json(table: ActorRef<Protocol>, s: &str) -> Chain {
        // @TODO inappropriate unwrap?
        // @see https://github.com/holochain/holochain-rust/issues/168
        let mut as_seq: Vec<Pair> = serde_json::from_str(s).unwrap();
        as_seq.reverse();

        let mut chain = Chain::new(table);

        for p in as_seq {
            chain.push_pair(&p).unwrap();
            // let response = chain.ask(ChainProtocol::PushPair(p));
            // let result = unwrap_to!(response => ChainProtocol::PushResult);
            // result.clone().unwrap();
        }
        chain
    }
}

impl SourceChain for Chain {
    /// returns a clone of the top Pair
    fn top_pair(&self) -> Option<Pair> {
        self.top_pair.clone()
    }

    /// get the top Pair by Entry type
    fn top_pair_type(&self, t: &str) -> Option<Pair> {
        self.iter().find(|p| p.header().entry_type() == t)
    }

    /// private pair-oriented version of push() (which expects Entries)
    fn push_pair(&mut self, pair: &Pair) -> Result<Pair, HolochainError> {
        if !(pair.validate()) {
            return Err(HolochainError::new(
                "attempted to push an invalid pair for this chain",
            ));
        }

        let top_pair = self.top_pair().and_then(|p| Some(p.key()));
        let next_pair = pair.header().next();

        if top_pair != next_pair {
            return Err(HolochainError::new(&format!(
                "top pair did not match next hash pair from pushed pair: {:?} vs. {:?}",
                top_pair.clone(),
                next_pair.clone()
            )));
        }

        let result = self.table.commit(&pair.clone());

        if result.is_ok() {
            self.top_pair = Some(pair.clone());
        }

        println!("after commit: {:?}", self);
        match result {
            Ok(_) => Ok(pair.clone()),
            Err(e) => Err(e.clone()),
        }
    }

    /// push a new Entry on to the top of the Chain
    /// the Pair for the new Entry is automatically generated and validated against the current top
    /// Pair to ensure the chain links up correctly across the underlying table data
    /// the newly created and pushed Pair is returned in the fn Result
    fn push_entry(&mut self, entry: &Entry) -> Result<Pair, HolochainError> {
        let pair = Pair::new(self, entry);
        self.push_pair(&pair)
    }

    /// get a Pair by Pair/Header key from the HashTable if it exists
    fn get_pair(&self, k: &str) -> Result<Option<Pair>, HolochainError> {
        let response = self.table.ask(Protocol::HashTableGetPair(k.to_string()));
        unwrap_to!(response => Protocol::HashTableGetPairResult).clone()
    }

    /// get an Entry by Entry key from the HashTable if it exists
    fn get_entry(&self, entry_hash: &str) -> Result<Option<Pair>, HolochainError> {
        println!("get entry: {:?}", entry_hash);
        // @TODO - this is a slow way to do a lookup
        // @see https://github.com/holochain/holochain-rust/issues/50
        Ok(self
                .iter()
                // @TODO entry hashes are NOT unique across pairs so k/v lookups can't be 1:1
                // @see https://github.com/holochain/holochain-rust/issues/145
                .find(|p| p.entry().hash() == entry_hash))
    }
}

pub trait SourceChain {
    fn top_pair(&self) -> Option<Pair>;
    fn top_pair_type(&self, t: &str) -> Option<Pair>;

    fn push_entry(&mut self, entry: &Entry) -> Result<Pair, HolochainError>;
    fn get_entry(&self, entry_hash: &str) -> Result<Option<Pair>, HolochainError>;

    fn push_pair(&mut self, pair: &Pair) -> Result<Pair, HolochainError>;
    fn get_pair(&self, message: &str) -> Result<Option<Pair>, HolochainError>;
}

#[cfg(test)]
pub mod tests {

    use super::Chain;
    use chain::SourceChain;
    use hash_table::{
        actor::tests::test_table_actor,
        entry::tests::{test_entry, test_entry_a, test_entry_b, test_type_a, test_type_b},
        pair::Pair,
        HashTable,
    };

    /// builds a dummy chain for testing
    pub fn test_chain() -> Chain {
        Chain::new(test_table_actor())
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

        c1.push_entry(&e1).unwrap();
        c2.push_entry(&e1).unwrap();
        c3.push_entry(&e2).unwrap();

        assert_eq!(c1.top_pair(), c2.top_pair());
        assert_eq!(c1, c2);

        assert_ne!(c1, c3);
        assert_ne!(c2, c3);
    }

    #[test]
    /// tests for chain.top()
    fn top() {
        let mut chain = test_chain();
        assert_eq!(None, chain.top_pair());

        let e1 = test_entry_a();
        let e2 = test_entry_b();

        let p1 = chain.push_entry(&e1).unwrap();
        assert_eq!(Some(p1), chain.top_pair());

        let p2 = chain.push_entry(&e2).unwrap();
        assert_eq!(Some(p2), chain.top_pair());
    }

    #[test]
    /// tests for chain.table()
    fn table_push() {
        let table_actor = test_table_actor();
        let mut chain = Chain::new(table_actor.clone());

        // test that adding something to the chain adds to the table
        let pair = chain.push_entry(&test_entry()).unwrap();

        assert_eq!(Some(pair.clone()), table_actor.get(&pair.key()).unwrap(),);
        assert_eq!(Some(pair.clone()), chain.get_pair(&pair.key()).unwrap(),);
        assert_eq!(
            table_actor.get(&pair.key()).unwrap(),
            chain.get_pair(&pair.key()).unwrap(),
        );
    }

    #[test]
    /// tests for chain.push()
    fn push() {
        let mut chain = test_chain();

        assert_eq!(None, chain.top_pair());

        // chain top, pair entry and headers should all line up after a push
        let e1 = test_entry_a();
        let p1 = chain.push_entry(&e1).unwrap();

        assert_eq!(Some(p1.clone()), chain.top_pair());
        assert_eq!(e1, p1.entry());
        assert_eq!(e1.hash(), p1.header().entry());

        // we should be able to do it again
        let e2 = test_entry_b();
        let p2 = chain.push_entry(&e2).unwrap();

        assert_eq!(Some(p2.clone()), chain.top_pair());
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

        chain.push_entry(&e1).unwrap();
        assert!(chain.validate());

        chain.push_entry(&e2).unwrap();
        assert!(chain.validate());
    }

    #[test]
    /// test chain.push() and chain.get() together
    fn round_trip() {
        let mut c = test_chain();
        let e = test_entry();
        let p = c.push_entry(&e).unwrap();
        assert_eq!(Some(p.clone()), c.get_pair(&p.key()).unwrap(),);
    }

    #[test]
    /// test chain.iter()
    fn iter() {
        let mut chain = test_chain();

        let e1 = test_entry_a();
        let e2 = test_entry_b();

        let p1 = chain.push_entry(&e1).unwrap();
        let p2 = chain.push_entry(&e2).unwrap();

        assert_eq!(vec![p2, p1], chain.iter().collect::<Vec<Pair>>());
    }

    #[test]
    /// test chain.iter() functional interface
    fn iter_functional() {
        let mut chain = test_chain();

        let e1 = test_entry_a();
        let e2 = test_entry_b();

        let p1 = chain.push_entry(&e1).unwrap();
        let _p2 = chain.push_entry(&e2).unwrap();
        let p3 = chain.push_entry(&e1).unwrap();

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

        let p1 = chain.push_entry(&e1).unwrap();
        let p2 = chain.push_entry(&e2).unwrap();
        let p3 = chain.push_entry(&e3).unwrap();

        assert_eq!(None, chain.get_pair("").unwrap());
        assert_eq!(Some(p1.clone()), chain.get_pair(&p1.key()).unwrap());
        assert_eq!(Some(p2.clone()), chain.get_pair(&p2.key()).unwrap());
        assert_eq!(Some(p3.clone()), chain.get_pair(&p3.key()).unwrap());
        assert_eq!(
            Some(p1.clone()),
            chain.get_pair(&p1.header().key()).unwrap()
        );
        assert_eq!(
            Some(p2.clone()),
            chain.get_pair(&p2.header().key()).unwrap()
        );
        assert_eq!(
            Some(p3.clone()),
            chain.get_pair(&p3.header().key()).unwrap()
        );
    }

    #[test]
    /// test chain.get_entry()
    fn get_entry() {
        let mut chain = test_chain();

        let e1 = test_entry_a();
        let e2 = test_entry_b();
        let e3 = test_entry_a();

        let p1 = chain.push_entry(&e1).unwrap();
        let p2 = chain.push_entry(&e2).unwrap();
        let p3 = chain.push_entry(&e3).unwrap();

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

        assert_eq!(None, chain.top_pair_type(&test_type_a()));
        assert_eq!(None, chain.top_pair_type(&test_type_b()));

        let e1 = test_entry_a();
        let e2 = test_entry_b();
        let e3 = test_entry_a();

        // type a should be p1
        // type b should be None
        let p1 = chain.push_entry(&e1).unwrap();
        assert_eq!(Some(p1.clone()), chain.top_pair_type(&test_type_a()));
        assert_eq!(None, chain.top_pair_type(&test_type_b()));

        // type a should still be p1
        // type b should be p2
        let p2 = chain.push_entry(&e2).unwrap();
        assert_eq!(Some(p1.clone()), chain.top_pair_type(&test_type_a()));
        assert_eq!(Some(p2.clone()), chain.top_pair_type(&test_type_b()));

        // type a should be p3
        // type b should still be p2
        let p3 = chain.push_entry(&e3).unwrap();
        assert_eq!(Some(p3.clone()), chain.top_pair_type(&test_type_a()));
        assert_eq!(Some(p2.clone()), chain.top_pair_type(&test_type_b()));
    }

    #[test]
    /// test IntoIterator implementation
    fn into_iter() {
        let mut chain = test_chain();

        let e1 = test_entry_a();
        let e2 = test_entry_b();
        let e3 = test_entry_a();

        let p1 = chain.push_entry(&e1).unwrap();
        let p2 = chain.push_entry(&e2).unwrap();
        let p3 = chain.push_entry(&e3).unwrap();

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

        chain.push_entry(&e1).unwrap();
        chain.push_entry(&e2).unwrap();
        chain.push_entry(&e3).unwrap();

        let expected_json = "[{\"header\":{\"entry_type\":\"testEntryType\",\"time\":\"\",\"next\":\"QmPT5HXvyv54Dg36YSK1A2rYvoPCNWoqpLzzZnHnQBcU6x\",\"entry\":\"QmbXSE38SN3SuJDmHKSSw5qWWegvU7oTxrLDRavWjyxMrT\",\"type_next\":\"QmawqBCVVap9KdaakqEHF4JzUjjLhmR7DpM5jgJko8j1rA\",\"signature\":\"\"},\"entry\":{\"content\":\"test entry content\",\"entry_type\":\"testEntryType\"}},{\"header\":{\"entry_type\":\"testEntryTypeB\",\"time\":\"\",\"next\":\"QmawqBCVVap9KdaakqEHF4JzUjjLhmR7DpM5jgJko8j1rA\",\"entry\":\"QmPz5jKXsxq7gPVAbPwx5gD2TqHfqB8n25feX5YH18JXrT\",\"type_next\":null,\"signature\":\"\"},\"entry\":{\"content\":\"other test entry content\",\"entry_type\":\"testEntryTypeB\"}},{\"header\":{\"entry_type\":\"testEntryType\",\"time\":\"\",\"next\":null,\"entry\":\"QmbXSE38SN3SuJDmHKSSw5qWWegvU7oTxrLDRavWjyxMrT\",\"type_next\":null,\"signature\":\"\"},\"entry\":{\"content\":\"test entry content\",\"entry_type\":\"testEntryType\"}}]";
        assert_eq!(expected_json, chain.to_json().unwrap());

        let table_actor = test_table_actor();
        assert_eq!(chain, Chain::from_json(table_actor, expected_json));
    }

}
