pub mod actor;
pub mod header;
pub mod pair;

use actor::Protocol;
use cas::content::{Address, AddressableContent};
use chain::{
    actor::{AskChain, ChainActor},
    header::ChainHeader,
    pair::Pair,
};
use error::HolochainError;
use hash_table::{
    entry::Entry,
    sys_entry::ToEntry,
    HashTable,
};
use holochain_dna::entry_type::EntryType;
use json::ToJson;
use riker::actors::*;
use serde_json;

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
                let header_entry = &self.table_actor.entry(&h)
                                    .expect("getting from a table shouldn't fail")
                                    .expect("getting from a table shouldn't fail");
                // Recreate the Pair from the ChainHeaderEntry
                let header = ChainHeader::from_entry(header_entry);
                let pair = Pair::from_header(&self.table_actor, &header);
                pair
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

    /// Create the next commitable ChainHeader for the chain.
    /// a ChainHeader is immutable, but the chain is mutable if chain.commit_*() is used.
    /// this means that a header becomes invalid and useless as soon as the chain is mutated
    /// the only valid usage of a header is to immediately commit it onto a chain in a Pair.
    /// normally (outside unit tests) the generation of valid headers is internal to the
    /// chain::SourceChain trait and should not need to be handled manually
    ///
    /// @see chain::pair::Pair
    /// @see chain::entry::Entry
    pub fn create_next_header(&self, entry_type: &EntryType, entry: &Entry) -> ChainHeader {
        ChainHeader::new(
            entry_type,
            // @TODO implement timestamps
            // https://github.com/holochain/holochain-rust/issues/70
            &String::new(),
            self.top_pair()
                .expect("could not get top pair when building header")
                .as_ref()
                .map(|p| p.header().to_entry().1.address()),
            &entry.address(),
            // @TODO implement signatures
            // https://github.com/holochain/holochain-rust/issues/71
            &String::new(),
            self
                .top_pair_of_type(entry_type)
                // @TODO inappropriate expect()?
                // @see https://github.com/holochain/holochain-rust/issues/147
                .map(|p| p.header().address()),
        )
    }

    /// Create the next commitable Pair for this chain
    ///
    /// ChainHeader is generated
    ///
    /// a Pair is immutable, but the chain is mutable if chain.commit_*() is used.
    ///
    /// this means that if two Pairs X and Y are generated for chain C then Pair X is pushed onto
    /// C to create chain C' (containing X), then Pair Y is no longer valid as the headers would
    /// need to include X. Pair Y can be regenerated with the same parameters as Y' and will be
    /// now be valid, the new Y' will include correct headers pointing to X.
    ///
    /// # Panics
    ///
    /// Panics if entry is somehow invalid
    ///
    /// @see chain::entry::Entry
    /// @see chain::header::ChainHeader
    pub fn create_next_pair(&self, entry_type: &EntryType, entry: &Entry) -> Pair {
        let new_pair = Pair::new(&self.create_next_header(entry_type, entry), &entry.clone());

        new_pair
    }

    /// returns a ChainIterator that provides cloned Pairs from the underlying HashTable
    fn iter(&self) -> ChainIterator {
        ChainIterator::new(
            self.table(),
            &self
                .top_pair()
                .expect("could not get top pair when building iterator"),
        )
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

    /// table getter
    /// returns a reference to the underlying HashTable
    pub fn table(&self) -> ActorRef<Protocol> {
        self.table_actor.clone()
    }
}

// @TODO should SourceChain have a bound on HashTable for consistency?
// @see https://github.com/holochain/holochain-rust/issues/261
pub trait SourceChain {
    /// sets an option for the top Pair
    fn set_top_pair(&self, &Option<Pair>) -> Result<Option<Pair>, HolochainError>;
    /// returns an option for the top Pair
    fn top_pair(&self) -> Result<Option<Pair>, HolochainError>;
    /// get the top Pair by Entry type
    fn top_pair_of_type(&self, entry_type: &EntryType) -> Option<Pair>;

    /// push a new Entry on to the top of the Chain.
    /// The Pair for the new Entry is generated and validated against the current top
    /// Pair to ensure the chain links up correctly across the underlying table data
    /// the newly created and pushed Pair is returned.
    fn push_entry(&mut self, entry_type: &EntryType, entry: &Entry)
        -> Result<Pair, HolochainError>;
    /// get an Entry by Entry address from the HashTable if it exists
    fn entry(&self, entry_address: &Address) -> Option<Entry>;

    /// pair-oriented version of push_entry()
    fn push_pair(&mut self, pair: &Pair) -> Result<Pair, HolochainError>;
    /// get a Pair by Pair/ChainHeader address from the HashTable if it exists
    fn pair(&self, pair_address: &Address) -> Option<Pair>;
}

impl SourceChain for Chain {
    fn top_pair(&self) -> Result<Option<Pair>, HolochainError> {
        self.chain_actor.top_pair()
    }

    fn set_top_pair(&self, pair: &Option<Pair>) -> Result<Option<Pair>, HolochainError> {
        self.chain_actor.set_top_pair(&pair)
    }

    fn top_pair_of_type(&self, entry_type: &EntryType) -> Option<Pair> {
        self.iter()
            .find(|pair| pair.header().entry_type() == entry_type)
    }

    fn push_pair(&mut self, pair: &Pair) -> Result<Pair, HolochainError> {
        let (_, header_entry) = &pair.clone().header().to_entry();
        self.table_actor.put_entry(header_entry)?;
        self.table_actor.put_entry(&pair.clone().entry())?;

        // @TODO if top pair set fails but commit succeeds?
        // @see https://github.com/holochain/holochain-rust/issues/259
        self.set_top_pair(&Some(pair.clone()))?;

        Ok(pair.clone())
    }

    fn push_entry(
        &mut self,
        entry_type: &EntryType,
        entry: &Entry,
    ) -> Result<Pair, HolochainError> {
        let pair = self.create_next_pair(entry_type, entry);
        self.push_pair(&pair)
    }

    /// Browse Chain until Pair is found
    fn pair(&self, pair_address: &Address) -> Option<Pair> {
        // @TODO - this is a slow way to do a lookup
        // @see https://github.com/holochain/holochain-rust/issues/50
        self
            .iter()
            // @TODO entry addresses are NOT unique across pairs so k/v lookups can't be 1:1
            // @see https://github.com/holochain/holochain-rust/issues/145
            .find(|pair| {
                &pair.address() == pair_address
            })
    }

    /// Browse Chain until Pair with entry_address is found
    fn entry(&self, entry_address: &Address) -> Option<Entry> {
        // @TODO - this is a slow way to do a lookup
        // @see https://github.com/holochain/holochain-rust/issues/50
        let pair = self
                .iter()
                // @TODO entry addresses are NOT unique across pairs so k/v lookups can't be 1:1
                // @see https://github.com/holochain/holochain-rust/issues/145
            .find(|pair| {
                &pair.entry().address() == entry_address
            });
        if pair.is_none() {
            return None;
        };
        Some(pair.unwrap().entry().clone())
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
    use cas::content::{Address, AddressableContent};
    use chain::{
        pair::{tests::test_pair, Pair},
        SourceChain,
    };
    use hash_table::{
        actor::tests::test_table_actor,
        entry::tests::{
            test_entry, test_entry_a, test_entry_b, test_entry_type, test_entry_type_a,
            test_entry_type_b,
        },
        HashTable,
    };
    use json::ToJson;
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

        let entry_type_a = test_entry_type_a();
        let entry_type_b = test_entry_type_b();

        let entry_a = test_entry_a();
        let entry_b = test_entry_b();

        chain1
            .push_entry(&entry_type_a, &entry_a)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");
        chain2
            .push_entry(&entry_type_a, &entry_a)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");
        chain3
            .push_entry(&entry_type_b, &entry_b)
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

        assert_eq!(
            None,
            chain
                .top_pair()
                .expect("could not get top pair from test chain")
        );

        let entry_type_a = test_entry_type_a();
        let entry_type_b = test_entry_type_b();

        let entry_a = test_entry_a();
        let entry_b = test_entry_b();

        let pair_a = chain
            .push_entry(&entry_type_a, &entry_a)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");
        assert_eq!(&entry_a, pair_a.entry());
        let top_pair = chain.top_pair().expect("should have commited entry");
        assert_eq!(Some(pair_a), top_pair);

        let pair_b = chain
            .push_entry(&entry_type_b, &entry_b)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");
        assert_eq!(&entry_b, pair_b.entry());
        let top_pair = chain.top_pair().expect("should have commited entry");
        assert_eq!(Some(pair_b), top_pair);
    }

    #[test]
    /// tests that the chain state is consistent across clones
    fn clone_safe() {
        let chain_1 = test_chain();
        let mut chain_2 = chain_1.clone();
        let test_pair = test_pair();

        assert_eq!(
            None,
            chain_1
                .top_pair()
                .expect("could not get top pair for chain 1")
        );
        assert_eq!(
            None,
            chain_2
                .top_pair()
                .expect("could not get top pair for chain 2")
        );

        let pair = chain_2.push_pair(&test_pair).unwrap();

        assert_eq!(
            Some(pair.clone()),
            chain_2
                .top_pair()
                .expect("could not get top pair after pushing to chain 2")
        );
        assert_eq!(
            chain_1
                .top_pair()
                .expect("could not get top pair for comparing chain 1"),
            chain_2
                .top_pair()
                .expect("could not get top pair when comparing chain 2")
        );
    }

    #[test]
    // test that adding something to the chain adds to the table
    fn table_put() {
        let table_actor = test_table_actor();
        let mut chain = Chain::new(table_actor.clone());

        let pair = chain
            .push_pair(&test_pair())
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");

        let table_entry = table_actor
            .entry(&pair.entry().address())
            .expect("getting an entry from a table in a chain shouldn't fail")
            .expect("table should have entry");
        let chain_entry = chain
            .entry(&pair.entry().address())
            .expect("getting an entry from a chain shouldn't fail");

        assert_eq!(pair.entry(), &table_entry);
        assert_eq!(table_entry, chain_entry);
    }

    #[test]
    fn can_commit_entry() {
        let mut chain = test_chain();

        assert_eq!(
            None,
            chain
                .top_pair()
                .expect("could not get top pair for test chain")
        );

        // chain top, pair entry and headers should all line up after a push
        let entry_type_a = test_entry_type_a();
        let entry_a = test_entry_a();
        let pair_a = chain
            .push_entry(&entry_type_a, &entry_a)
            .expect("pushing a valid entry to an exclusively owned chain shouldn't fail");

        assert_eq!(
            Some(&pair_a),
            chain
                .top_pair()
                .expect("could not get top pair for pair a")
                .as_ref()
        );
        assert_eq!(&entry_a, pair_a.entry());
        assert_eq!(entry_a.address(), pair_a.entry().address());

        // we should be able to do it again
        let entry_type_b = test_entry_type_b();
        let entry_b = test_entry_b();
        let pair_b = chain
            .push_entry(&entry_type_b, &entry_b)
            .expect("pushing a valid entry to an exclusively owned chain shouldn't fail");

        assert_eq!(
            Some(&pair_b),
            chain
                .top_pair()
                .expect("could not get top pair for pair 2")
                .as_ref()
        );
        assert_eq!(&entry_b, pair_b.entry());
        assert_eq!(entry_b.address(), pair_b.entry().address());
    }

    #[test]
    /// test chain.push() and chain.get() together
    fn round_trip() {
        let mut chain = test_chain();
        let entry_type = test_entry_type();
        let entry = test_entry();
        let pair = chain
            .push_entry(&entry_type, &entry)
            .expect("pushing a valid entry to an exclusively owned chain shouldn't fail");

        assert_eq!(
            entry,
            chain
                .entry(&pair.entry().address())
                .expect("getting an entry from a chain shouldn't fail"),
        );
    }

    #[test]
    /// show that we can push the chain a bit without issues e.g. async
    fn round_trip_stress_test() {
        let h = thread::spawn(|| {
            let mut chain = test_chain();
            let entry_type = test_entry_type();
            let entry = test_entry();

            for _ in 1..100 {
                let pair = chain.push_entry(&entry_type, &entry).unwrap();
                assert_eq!(
                    Some(pair.entry().clone()),
                    chain.entry(&pair.entry().address()),
                );
            }
        });
        h.join().unwrap();
    }

    #[test]
    /// test chain.iter()
    fn iter() {
        let mut chain = test_chain();

        let entry_type_a = test_entry_type_a();
        let entry_type_b = test_entry_type_b();

        let entry_a = test_entry_a();
        let entry_b = test_entry_b();

        let pair_a = chain
            .push_entry(&entry_type_a, &entry_a)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");
        let pair_b = chain
            .push_entry(&entry_type_b, &entry_b)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");

        assert_eq!(vec![pair_b, pair_a], chain.iter().collect::<Vec<Pair>>());
    }

    #[test]
    /// test chain.iter() functional interface
    fn iter_functional() {
        let mut chain = test_chain();

        let entry_type_a = test_entry_type_a();
        let entry_type_b = test_entry_type_b();

        let entry_a = test_entry_a();
        let entry_b = test_entry_b();

        let pair_a = chain
            .push_entry(&entry_type_a, &entry_a)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");
        let _pair_b = chain
            .push_entry(&entry_type_b, &entry_b)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");
        let pair_c = chain
            .push_entry(&entry_type_a, &entry_a)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");

        assert_eq!(
            vec![pair_c, pair_a],
            chain
                .iter()
                .filter(|pair| pair.header().entry_type() == &entry_type_a)
                .collect::<Vec<Pair>>()
        );
    }

    #[test]
    fn entry_advance() {
        let mut chain = test_chain();

        let entry_type_a = test_entry_type_a();
        let entry_type_b = test_entry_type_b();

        let entry_a = test_entry_a();
        let entry_b = test_entry_b();

        let pair_a = chain
            .push_entry(&entry_type_a, &entry_a)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");
        let pair_b = chain
            .push_entry(&entry_type_b, &entry_b)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");

        assert_eq!(
            pair_a.entry().clone(),
            chain
                .entry(&pair_a.entry().address())
                .expect("getting an entry from a chain shouldn't fail"),
        );

        let pair_c = chain
            .push_entry(&entry_type_a, &entry_a)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");

        assert_eq!(None, chain.entry(&Address::new()));
        assert_eq!(
            pair_c.entry().clone(),
            chain
                .entry(&pair_a.entry().address())
                .expect("getting an entry from a chain shouldn't fail"),
        );
        assert_eq!(
            pair_b.entry().clone(),
            chain
                .entry(&pair_b.entry().address())
                .expect("getting an entry from a chain shouldn't fail"),
        );
        assert_eq!(
            pair_c.entry().clone(),
            chain
                .entry(&pair_c.entry().address())
                .expect("getting an entry from a chain shouldn't fail"),
        );

        assert_eq!(
            pair_a,
            chain
                .pair(&pair_a.address())
                .expect("getting an entry from a chain shouldn't fail"),
        );
        assert_eq!(
            pair_b,
            chain
                .pair(&pair_b.address())
                .expect("getting an entry from a chain shouldn't fail"),
        );
        assert_eq!(
            pair_c,
            chain
                .pair(&pair_c.address())
                .expect("getting an entry from a chain shouldn't fail"),
        );
    }

    #[test]
    fn entry() {
        let mut chain = test_chain();

        let entry_type_a = test_entry_type_a();
        let entry_type_b = test_entry_type_b();

        let entry_a = test_entry_a();
        let entry_b = test_entry_b();

        let pair_a = chain
            .push_entry(&entry_type_a, &entry_a)
            .expect("pushing a valid entry to an exclusively owned chain shouldn't fail");
        let pair_b = chain
            .push_entry(&entry_type_b, &entry_b)
            .expect("pushing a valid entry to an exclusively owned chain shouldn't fail");
        let pair_c = chain
            .push_entry(&entry_type_a, &entry_a)
            .expect("pushing a valid entry to an exclusively owned chain shouldn't fail");

        assert_eq!(None, chain.entry(&Address::new()));
        // @TODO at this point we have p3 with the same entry key as p1...
        assert_eq!(
            pair_c.entry().clone(),
            chain
                .entry(&pair_a.entry().address())
                .expect("getting an entry from a chain shouldn't fail"),
        );
        assert_eq!(
            pair_b.entry().clone(),
            chain
                .entry(&pair_b.entry().address())
                .expect("getting an entry from a chain shouldn't fail"),
        );
        assert_eq!(
            pair_c.entry().clone(),
            chain
                .entry(&pair_c.entry().address())
                .expect("getting an entry from a chain shouldn't fail"),
        );
    }

    #[test]
    fn top_pair_of_type() {
        let mut chain = test_chain();

        assert_eq!(None, chain.top_pair_of_type(&test_entry_type_a()));
        assert_eq!(None, chain.top_pair_of_type(&test_entry_type_b()));

        let entry_type_a = test_entry_type_a();
        let entry_type_b = test_entry_type_b();

        let entry_a = test_entry_a();
        let entry_b = test_entry_b();

        // type a should be pair_a
        // type b should be None
        let pair_a = chain
            .push_entry(&entry_type_a, &entry_a)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");
        assert_eq!(
            Some(&pair_a),
            chain.top_pair_of_type(&entry_type_a).as_ref()
        );
        assert_eq!(None, chain.top_pair_of_type(&entry_type_b));

        // type a should still be pair_a
        // type b should be pair_b
        let pair_b = chain
            .push_entry(&entry_type_b, &entry_b)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");
        assert_eq!(
            Some(&pair_a),
            chain.top_pair_of_type(&test_entry_type_a()).as_ref()
        );
        assert_eq!(
            Some(&pair_b),
            chain.top_pair_of_type(&test_entry_type_b()).as_ref()
        );

        // type a should be pair3
        // type b should still be pair2
        let pair_c = chain
            .push_entry(&entry_type_a, &entry_a)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");

        assert_eq!(
            Some(&pair_c),
            chain.top_pair_of_type(&test_entry_type_a()).as_ref()
        );
        assert_eq!(
            Some(&pair_b),
            chain.top_pair_of_type(&test_entry_type_b()).as_ref()
        );
    }

    #[test]
    /// test IntoIterator implementation
    fn into_iter() {
        let mut chain = test_chain();

        let entry_type_a = test_entry_type_a();
        let entry_type_b = test_entry_type_b();

        let entry_a = test_entry_a();
        let entry_b = test_entry_b();

        let pair_a = chain
            .push_entry(&entry_type_a, &entry_a)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");
        let pair_b = chain
            .push_entry(&entry_type_b, &entry_b)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");
        let pair_c = chain
            .push_entry(&entry_type_a, &entry_a)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");

        // into_iter() returns clones of pairs
        assert_eq!(
            vec![pair_c, pair_b, pair_a],
            chain.into_iter().collect::<Vec<Pair>>()
        );
    }

    #[test]
    /// test to_json() and from_json() implementation
    fn json_round_trip() {
        let mut chain = test_chain();

        let entry_type_a = test_entry_type_a();
        let entry_type_b = test_entry_type_b();

        let entry_a = test_entry_a();
        let entry_b = test_entry_b();

        chain
            .push_entry(&entry_type_a, &entry_a)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");
        chain
            .push_entry(&entry_type_b, &entry_b)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");
        chain
            .push_entry(&entry_type_a, &entry_a)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");

        let expected_json = "[{\"header\":{\"entry_type\":{\"App\":\"testEntryType\"},\"timestamp\":\"\",\"link\":\"QmR1XSoMwvjoiLG6NC7Zw3iy6cnfQsxjM5bt32thaCGbNU\",\"entry_address\":\"QmbXSE38SN3SuJDmHKSSw5qWWegvU7oTxrLDRavWjyxMrT\",\"entry_signature\":\"\",\"link_same_type\":\"Qmc1n5gbUU2QKW6is9ENTqmaTcEjYMBwNkcACCxe3bBDnd\"},\"entry\":\"test entry content\"},{\"header\":{\"entry_type\":{\"App\":\"testEntryTypeB\"},\"timestamp\":\"\",\"link\":\"Qmc1n5gbUU2QKW6is9ENTqmaTcEjYMBwNkcACCxe3bBDnd\",\"entry_address\":\"QmPz5jKXsxq7gPVAbPwx5gD2TqHfqB8n25feX5YH18JXrT\",\"entry_signature\":\"\",\"link_same_type\":null},\"entry\":\"other test entry content\"},{\"header\":{\"entry_type\":{\"App\":\"testEntryType\"},\"timestamp\":\"\",\"link\":null,\"entry_address\":\"QmbXSE38SN3SuJDmHKSSw5qWWegvU7oTxrLDRavWjyxMrT\",\"entry_signature\":\"\",\"link_same_type\":null},\"entry\":\"test entry content\"}]"
        ;
        assert_eq!(
            expected_json,
            chain.to_json().expect("chain shouldn't fail to serialize")
        );

        let table_actor = test_table_actor();
        assert_eq!(chain, Chain::from_json(table_actor, expected_json));
    }

}
