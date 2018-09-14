pub mod actor;

use actor::{AskSelf, Protocol};
use chain::actor::{AskChain, ChainActor};
use error::HolochainError;
use hash_table::{entry::Entry, pair::Pair, HashTable};
use json::ToJson;
use key::Key;
use riker::actors::*;
use serde_json;
pub mod header;

/// Iterator type for pairs in a chain
/// next method may panic if there is an error in the underlying table
#[derive(Clone)]
pub struct ChainIterator {
    table_actor: ActorRef<Protocol>,
    current: Option<Pair>,
}

impl ChainIterator {
    #[allow(unknown_lints)]
    #[allow(needless_pass_by_value)]
    pub fn new(table_actor: ActorRef<Protocol>, pair: &Option<Pair>) -> ChainIterator {
        ChainIterator {
            current: pair.clone(),
            table_actor: table_actor.clone(),
        }
    }
}

impl Iterator for ChainIterator {
    type Item = Pair;

    /// May panic if there is an underlying error in the table
    fn next(&mut self) -> Option<Pair> {
        let previous = self.current.take();
        self.current = previous
            .as_ref()
            .and_then(|p| p.header().link())
            // @TODO should this panic?
            // @see https://github.com/holochain/holochain-rust/issues/146
            .and_then(|h| {
                self.table_actor
                    .pair(&h.to_string())
                    .expect("getting from a table shouldn't fail")
            });
        previous
    }
}

#[derive(Clone, Debug)]
pub struct Chain {
    chain_actor: ActorRef<Protocol>,
    table_actor: ActorRef<Protocol>,
}

impl PartialEq for Chain {
    // @TODO can we just check the actors are equal? is actor equality a thing?
    // @see https://github.com/holochain/holochain-rust/issues/257
    fn eq(&self, other: &Chain) -> bool {
        // an invalid chain is like NaN... not even equal to itself
        self.validate() &&
        other.validate() &&
        // header hashing ensures that if the tops match the whole chain matches
        self.top_pair() == other.top_pair()
    }
}

impl Eq for Chain {}

/// Turns a chain into an iterator over it's Pairs
impl IntoIterator for Chain {
    type Item = Pair;
    type IntoIter = ChainIterator;

    /// returns a ChainIterator that provides cloned Pairs from the underlying HashTable
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl Chain {
    pub fn new(table: ActorRef<Protocol>) -> Chain {
        Chain {
            chain_actor: ChainActor::new_ref(),
            table_actor: table.clone(),
        }
    }

    /// returns a reference to the underlying HashTable
    pub fn table(&self) -> ActorRef<Protocol> {
        self.table_actor.clone()
    }

    /// returns true if all pairs in the chain pass validation
    fn validate(&self) -> bool {
        self.iter().all(|p| p.validate())
    }

    /// returns a ChainIterator that provides cloned Pairs from the underlying HashTable
    fn iter(&self) -> ChainIterator {
        ChainIterator::new(self.table(), &self.top_pair())
    }

    /// restore canonical JSON chain
    /// can't implement json::FromJson due to Chain's need for a table actor
    /// @TODO accept canonical JSON
    /// @see https://github.com/holochain/holochain-rust/issues/75
    pub fn from_json(table: ActorRef<Protocol>, s: &str) -> Self {
        // @TODO inappropriate unwrap?
        // @see https://github.com/holochain/holochain-rust/issues/168
        let mut as_seq: Vec<Pair> = serde_json::from_str(s).expect("argument should be valid json");
        as_seq.reverse();

        let mut chain = Chain::new(table);

        for p in as_seq {
            chain.push_pair(&p).expect("pair should be valid");
        }
        chain
    }
}

// @TODO should SourceChain have a bound on HashTable for consistency?
// @see https://github.com/holochain/holochain-rust/issues/261
pub trait SourceChain {
    /// sets an option for the top Pair
    fn set_top_pair(&self, &Option<Pair>) -> Result<Option<Pair>, HolochainError>;
    /// returns an option for the top Pair
    fn top_pair(&self) -> Option<Pair>;
    /// get the top Pair by Entry type
    fn top_pair_type(&self, t: &str) -> Option<Pair>;

    /// push a new Entry on to the top of the Chain
    /// the Pair for the new Entry is automatically generated and validated against the current top
    /// Pair to ensure the chain links up correctly across the underlying table data
    /// the newly created and pushed Pair is returned in the fn Result
    fn push_entry(&mut self, entry: &Entry) -> Result<Pair, HolochainError>;
    /// get an Entry by Entry key from the HashTable if it exists
    fn entry(&self, entry_hash: &str) -> Result<Option<Pair>, HolochainError>;

    /// pair-oriented version of push_entry()
    fn push_pair(&mut self, pair: &Pair) -> Result<Pair, HolochainError>;
    /// get a Pair by Pair/Header key from the HashTable if it exists
    fn pair(&self, message: &str) -> Result<Option<Pair>, HolochainError>;
}

impl SourceChain for Chain {
    fn top_pair(&self) -> Option<Pair> {
        self.chain_actor.top_pair()
    }

    fn set_top_pair(&self, pair: &Option<Pair>) -> Result<Option<Pair>, HolochainError> {
        match pair {
            Some(pair_for_validation) => {
                if !(pair_for_validation.validate()) {
                    return Err(HolochainError::new(
                        "attempted to push an invalid pair for this chain",
                    ));
                }

                let top_pair = self.top_pair().as_ref().map(|p| p.key());
                let next_pair = pair_for_validation.header().link();

                if top_pair != next_pair {
                    return Err(HolochainError::new(&format!(
                        "top pair did not match next hash pair from pushed pair: {:?} vs. {:?}",
                        top_pair, next_pair,
                    )));
                }
                self.chain_actor.set_top_pair(&pair)
            }
            None => Ok(None),
        }
    }

    fn top_pair_type(&self, t: &str) -> Option<Pair> {
        self.iter().find(|p| p.header().entry_type() == t)
    }

    fn push_pair(&mut self, pair: &Pair) -> Result<Pair, HolochainError> {
        self.table_actor.put_pair(&pair.clone())?;

        // @TODO if top pair set fails but commit succeeds?
        // @see https://github.com/holochain/holochain-rust/issues/259
        self.set_top_pair(&Some(pair.clone()))?;

        Ok(pair.clone())
    }

    fn push_entry(&mut self, entry: &Entry) -> Result<Pair, HolochainError> {
        let pair = Pair::new(self, entry);
        self.push_pair(&pair)
    }

    fn pair(&self, k: &str) -> Result<Option<Pair>, HolochainError> {
        let response = self
            .table_actor
            .block_on_ask(Protocol::GetPair(k.to_string()));
        unwrap_to!(response => Protocol::GetPairResult).clone()
    }

    fn entry(&self, entry_hash: &str) -> Result<Option<Pair>, HolochainError> {
        // @TODO - this is a slow way to do a lookup
        // @see https://github.com/holochain/holochain-rust/issues/50
        Ok(self
            .iter()
            // @TODO entry hashes are NOT unique across pairs so k/v lookups can't be 1:1
            // @see https://github.com/holochain/holochain-rust/issues/145
            .find(|p| p.entry().hash() == entry_hash))
    }
}

impl ToJson for Chain {
    /// get the entire chain, top to bottom as a JSON array or canonical pairs
    /// @TODO return canonical JSON
    /// @see https://github.com/holochain/holochain-rust/issues/75
    fn to_json(&self) -> Result<String, HolochainError> {
        let as_seq = self.iter().collect::<Vec<Pair>>();
        Ok(serde_json::to_string(&as_seq)?)
    }
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
    use json::ToJson;
    use key::Key;
    use std::thread;

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
        let mut chain1 = test_chain();
        let mut chain2 = test_chain();
        let mut chain3 = test_chain();

        let entry_a = test_entry_a();
        let entry_b = test_entry_b();

        chain1
            .push_entry(&entry_a)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");
        chain2
            .push_entry(&entry_a)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");
        chain3
            .push_entry(&entry_b)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");

        assert_eq!(chain1.top_pair(), chain2.top_pair());
        assert_eq!(chain1, chain2);

        assert_ne!(chain1, chain3);
        assert_ne!(chain2, chain3);
    }

    #[test]
    /// tests for chain.top_pair()
    fn top_pair() {
        let mut chain = test_chain();

        assert_eq!(None, chain.top_pair());

        let entry_a = test_entry_a();
        let entry_b = test_entry_b();

        let pair_a = chain
            .push_entry(&entry_a)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");
        assert_eq!(Some(pair_a), chain.top_pair());

        let pair_b = chain
            .push_entry(&entry_b)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");
        assert_eq!(Some(pair_b), chain.top_pair());
    }

    #[test]
    /// tests that the chain state is consistent across clones
    fn clone_safe() {
        let c1 = test_chain();
        let mut c2 = c1.clone();
        let e = test_entry();

        assert_eq!(None, c1.top_pair());
        assert_eq!(None, c2.top_pair());

        let pair = c2.push_entry(&e).unwrap();

        assert_eq!(Some(pair.clone()), c2.top_pair());
        assert_eq!(c1.top_pair(), c2.top_pair());
    }

    #[test]
    /// tests for chain.table()
    fn table_push() {
        let table_actor = test_table_actor();
        let mut chain = Chain::new(table_actor.clone());

        // test that adding something to the chain adds to the table
        let pair = chain
            .push_entry(&test_entry())
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");

        let table_pair = table_actor
            .pair(&pair.key())
            .expect("getting an entry from a table in a chain shouldn't fail");
        let chain_pair = chain
            .pair(&pair.key())
            .expect("getting an entry from a chain shouldn't fail");

        assert_eq!(Some(&pair), table_pair.as_ref());
        assert_eq!(Some(&pair), chain_pair.as_ref());
        assert_eq!(table_pair, chain_pair);
    }

    #[test]
    /// tests for chain.push()
    fn push() {
        let mut chain = test_chain();

        assert_eq!(None, chain.top_pair());

        // chain top, pair entry and headers should all line up after a push
        let e1 = test_entry_a();
        let p1 = chain
            .push_entry(&e1)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");

        assert_eq!(Some(&p1), chain.top_pair().as_ref());
        assert_eq!(&e1, p1.entry());
        assert_eq!(e1.hash(), p1.header().entry_hash());

        // we should be able to do it again
        let e2 = test_entry_b();
        let p2 = chain
            .push_entry(&e2)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");

        assert_eq!(Some(&p2), chain.top_pair().as_ref());
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
        let mut chain = test_chain();
        let entry = test_entry();
        let pair = chain
            .push_entry(&entry)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");

        assert_eq!(
            Some(&pair),
            chain
                .pair(&pair.key())
                .expect("getting an entry from a chain shouldn't fail")
                .as_ref()
        );
    }

    #[test]
    /// show that we can push the chain a bit without issues e.g. async
    fn round_trip_stress_test() {
        let h = thread::spawn(|| {
            let mut chain = test_chain();
            let entry = test_entry();

            for _ in 1..100 {
                let pair = chain.push_entry(&entry).unwrap();
                assert_eq!(Some(pair.clone()), chain.pair(&pair.key()).unwrap(),);
            }
        });
        h.join().unwrap();
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
                .pair("")
                .expect("getting an entry from a chain shouldn't fail")
        );
        assert_eq!(
            Some(&p1),
            chain
                .pair(&p1.key())
                .expect("getting an entry from a chain shouldn't fail")
                .as_ref()
        );
        assert_eq!(
            Some(&p2),
            chain
                .pair(&p2.key())
                .expect("getting an entry from a chain shouldn't fail")
                .as_ref()
        );
        assert_eq!(
            Some(&p3),
            chain
                .pair(&p3.key())
                .expect("getting an entry from a chain shouldn't fail")
                .as_ref()
        );

        assert_eq!(
            Some(&p1),
            chain
                .pair(&p1.header().key())
                .expect("getting an entry from a chain shouldn't fail")
                .as_ref()
        );
        assert_eq!(
            Some(&p2),
            chain
                .pair(&p2.header().key())
                .expect("getting an entry from a chain shouldn't fail")
                .as_ref()
        );
        assert_eq!(
            Some(&p3),
            chain
                .pair(&p3.header().key())
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
                .entry("")
                .expect("getting an entry from a chain shouldn't fail")
        );
        // @TODO at this point we have p3 with the same entry key as p1...
        assert_eq!(
            Some(&p3),
            chain
                .entry(&p1.entry().key())
                .expect("getting an entry from a chain shouldn't fail")
                .as_ref()
        );
        assert_eq!(
            Some(&p2),
            chain
                .entry(&p2.entry().key())
                .expect("getting an entry from a chain shouldn't fail")
                .as_ref()
        );
        assert_eq!(
            Some(&p3),
            chain
                .entry(&p3.entry().key())
                .expect("getting an entry from a chain shouldn't fail")
                .as_ref()
        );
    }

    #[test]
    /// test chain.top_type()
    fn top_type() {
        let mut chain = test_chain();

        assert_eq!(None, chain.top_pair_type(&test_type_a()));
        assert_eq!(None, chain.top_pair_type(&test_type_b()));

        let entry1 = test_entry_a();
        let entry2 = test_entry_b();
        let entry3 = test_entry_a();

        // type a should be p1
        // type b should be None
        let pair1 = chain
            .push_entry(&entry1)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");
        assert_eq!(Some(&pair1), chain.top_pair_type(&test_type_a()).as_ref());
        assert_eq!(None, chain.top_pair_type(&test_type_b()));

        // type a should still be pair1
        // type b should be p2
        let pair2 = chain
            .push_entry(&entry2)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");
        assert_eq!(Some(&pair1), chain.top_pair_type(&test_type_a()).as_ref());
        assert_eq!(Some(&pair2), chain.top_pair_type(&test_type_b()).as_ref());

        // type a should be pair3
        // type b should still be pair2
        let pair3 = chain
            .push_entry(&entry3)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");

        assert_eq!(Some(&pair3), chain.top_pair_type(&test_type_a()).as_ref());
        assert_eq!(Some(&pair2), chain.top_pair_type(&test_type_b()).as_ref());
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

        let expected_json = "[{\"header\":{\"entry_type\":\"testEntryType\",\"timestamp\":\"\",\"link\":\"QmPT5HXvyv54Dg36YSK1A2rYvoPCNWoqpLzzZnHnQBcU6x\",\"entry_hash\":\"QmbXSE38SN3SuJDmHKSSw5qWWegvU7oTxrLDRavWjyxMrT\",\"entry_signature\":\"\",\"link_same_type\":\"QmawqBCVVap9KdaakqEHF4JzUjjLhmR7DpM5jgJko8j1rA\"},\"entry\":{\"content\":\"test entry content\",\"entry_type\":\"testEntryType\"}},{\"header\":{\"entry_type\":\"testEntryTypeB\",\"timestamp\":\"\",\"link\":\"QmawqBCVVap9KdaakqEHF4JzUjjLhmR7DpM5jgJko8j1rA\",\"entry_hash\":\"QmPz5jKXsxq7gPVAbPwx5gD2TqHfqB8n25feX5YH18JXrT\",\"entry_signature\":\"\",\"link_same_type\":null},\"entry\":{\"content\":\"other test entry content\",\"entry_type\":\"testEntryTypeB\"}},{\"header\":{\"entry_type\":\"testEntryType\",\"timestamp\":\"\",\"link\":null,\"entry_hash\":\"QmbXSE38SN3SuJDmHKSSw5qWWegvU7oTxrLDRavWjyxMrT\",\"entry_signature\":\"\",\"link_same_type\":null},\"entry\":{\"content\":\"test entry content\",\"entry_type\":\"testEntryType\"}}]"
        ;
        assert_eq!(
            expected_json,
            chain.to_json().expect("chain shouldn't fail to serialize")
        );

        let table_actor = test_table_actor();
        assert_eq!(chain, Chain::from_json(table_actor, expected_json));
    }

}
