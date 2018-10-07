pub mod actor;

use holochain_cas_implementations::actor::Protocol;
use holochain_core_types::{
    cas::content::{Address, AddressableContent},
    chain_header::ChainHeader,
    entry::Entry,
    entry_type::EntryType,
    error::HolochainError,
    json::ToJson,
    to_entry::ToEntry,
};
use chain::{
    actor::{AskChain, ChainActor},
};
use hash_table::HashTable;
use riker::actors::*;
use serde_json;

/// Iterator type for chain headers in a chain
/// next method may panic if there is an error in the underlying table
#[derive(Clone)]
pub struct ChainIterator {
    table_actor: ActorRef<Protocol>,
    current: Option<ChainHeader>,
}

impl ChainIterator {
    #[allow(unknown_lints)]
    #[allow(needless_pass_by_value)]
    pub fn new(
        table_actor: ActorRef<Protocol>,
        chain_header: &Option<ChainHeader>,
    ) -> ChainIterator {
        ChainIterator {
            current: chain_header.clone(),
            table_actor: table_actor.clone(),
        }
    }
}

impl Iterator for ChainIterator {
    type Item = ChainHeader;

    /// May panic if there is an underlying error in the table
    fn next(&mut self) -> Option<ChainHeader> {
        let previous = self.current.take();

        self.current = previous
            .as_ref()
            .and_then(|chain_header| chain_header.link())
            // @TODO should this panic?
            // @see https://github.com/holochain/holochain-rust/issues/146
            .and_then(|linked_chain_header_address| {
                let linked_header_entry = &self.table_actor.entry(&linked_chain_header_address)
                                    .expect("getting from a table shouldn't fail")
                                    .expect("getting from a table shouldn't fail");
                Some(ChainHeader::from_entry(linked_header_entry))
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
        self.top_chain_header() == other.top_chain_header()
    }
}

impl Eq for Chain {}

/// Turns a chain into an iterator over it's Pairs
impl IntoIterator for Chain {
    type Item = ChainHeader;
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
    /// the only valid usage of a header is to immediately commit it onto a chain in a ChainHeader.
    /// normally (outside unit tests) the generation of valid headers is internal to the
    /// chain::SourceChain trait and should not need to be handled manually
    ///
    /// @see chain::header::ChainHeader
    /// @see chain::entry::Entry
    pub fn create_next_chain_header(&self, entry_type: &EntryType, entry: &Entry) -> ChainHeader {
        ChainHeader::new(
            entry_type,
            // @TODO implement timestamps
            // https://github.com/holochain/holochain-rust/issues/70
            &String::new(),
            self.top_chain_header()
                .expect("could not get top chain header when building new header")
                .as_ref()
                .map(|chain_header| chain_header.to_entry().1.address()),
            &entry.address(),
            // @TODO implement signatures
            // https://github.com/holochain/holochain-rust/issues/71
            &String::new(),
            self.top_chain_header_of_type(entry_type)
                .map(|chain_header| chain_header.address()),
        )
    }

    /// returns a ChainIterator that provides cloned Pairs from the underlying HashTable
    fn iter(&self) -> ChainIterator {
        ChainIterator::new(
            self.table(),
            &self
                .top_chain_header()
                .expect("could not get top chain header when building iterator"),
        )
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
    fn set_top_chain_header(
        &self,
        &Option<ChainHeader>,
    ) -> Result<Option<ChainHeader>, HolochainError>;
    /// returns an option for the top Pair
    fn top_chain_header(&self) -> Result<Option<ChainHeader>, HolochainError>;
    /// get the top Pair by Entry type
    fn top_chain_header_of_type(&self, entry_type: &EntryType) -> Option<ChainHeader>;

    /// push a new Entry on to the top of the Chain.
    /// The ChainHeader for the new Entry is generated and validated against the current top
    /// ChainHeader to ensure the chain links up correctly across the underlying table data
    /// the newly created and pushed ChainHeader is returned.
    fn push_entry(
        &mut self,
        entry_type: &EntryType,
        entry: &Entry,
    ) -> Result<ChainHeader, HolochainError>;
    /// get an Entry by Entry address from the HashTable if it exists
    fn entry(&self, entry_address: &Address) -> Option<Entry>;
}

impl SourceChain for Chain {
    fn top_chain_header(&self) -> Result<Option<ChainHeader>, HolochainError> {
        self.chain_actor.top_chain_header()
    }

    fn set_top_chain_header(
        &self,
        chain_header: &Option<ChainHeader>,
    ) -> Result<Option<ChainHeader>, HolochainError> {
        self.chain_actor.set_top_chain_header(&chain_header)
    }

    fn top_chain_header_of_type(&self, entry_type: &EntryType) -> Option<ChainHeader> {
        self.iter()
            .find(|chain_header| chain_header.entry_type() == entry_type)
    }

    fn push_entry(
        &mut self,
        entry_type: &EntryType,
        entry: &Entry,
    ) -> Result<ChainHeader, HolochainError> {
        // entry first...
        self.table_actor.put_entry(entry)?;

        // then header...
        let chain_header = self.create_next_chain_header(entry_type, entry);
        self.table_actor.put_entry(&chain_header.to_entry().1)?;
        self.set_top_chain_header(&Some(chain_header.clone()))?;
        Ok(chain_header)
    }

    /// Browse Chain until Entry with entry_address is found
    fn entry(&self, entry_address: &Address) -> Option<Entry> {
        // @TODO - this is a slow way to do a lookup
        // @see https://github.com/holochain/holochain-rust/issues/50
        let maybe_chain_header = self
                .iter()
                // @TODO entry addresses are NOT unique across chain headers so k/v lookups can't
                // be 1:1
                // @see https://github.com/holochain/holochain-rust/issues/145
            .find(|chain_header| {
                &chain_header.entry_address() == &entry_address
            });
        maybe_chain_header.and_then(|chain_header| {
            self.table_actor
                .entry(chain_header.entry_address())
                .expect("failed to retrieve Entry from table actor")
        })
    }
}

impl ToJson for Chain {
    /// get the entire chain, top to bottom as a JSON array or canonical chain headers
    /// @TODO return canonical JSON
    /// @see https://github.com/holochain/holochain-rust/issues/75
    fn to_json(&self) -> Result<String, HolochainError> {
        let as_seq = self.iter().collect::<Vec<ChainHeader>>();
        Ok(serde_json::to_string(&as_seq)?)
    }
}

#[cfg(test)]
pub mod tests {

    use super::Chain;
    use holochain_core_types::{
        cas::content::{Address, AddressableContent},
        chain_header::ChainHeader,
        entry::{
            test_entry, test_entry_a, test_entry_b, test_entry_type, test_entry_type_a,
            test_entry_type_b,
        },
    };
    use chain::{SourceChain};
    use hash_table::{
        actor::tests::test_table_actor,
        HashTable,
    };
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

        assert_eq!(chain1.top_chain_header(), chain2.top_chain_header());
        assert_eq!(chain1, chain2);

        assert_ne!(chain1, chain3);
        assert_ne!(chain2, chain3);
    }

    #[test]
    /// tests for chain.top_chain_header()
    fn top_chain_header() {
        let mut chain = test_chain();

        assert_eq!(
            None,
            chain
                .top_chain_header()
                .expect("could not get top chain header from test chain")
        );

        let entry_type_a = test_entry_type_a();
        let entry_type_b = test_entry_type_b();

        let entry_a = test_entry_a();
        let entry_b = test_entry_b();

        let chain_header_a = chain
            .push_entry(&entry_type_a, &entry_a)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");
        assert_eq!(&entry_a.address(), chain_header_a.entry_address());
        let top_chain_header = chain
            .top_chain_header()
            .expect("should have commited entry");
        assert_eq!(Some(chain_header_a), top_chain_header);

        let chain_header_b = chain
            .push_entry(&entry_type_b, &entry_b)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");
        assert_eq!(&entry_b.address(), chain_header_b.entry_address());
        let top_chain_header = chain
            .top_chain_header()
            .expect("should have commited entry");
        assert_eq!(Some(chain_header_b), top_chain_header);
    }

    #[test]
    /// tests that the chain state is consistent across clones
    fn clone_safe() {
        let chain_1 = test_chain();
        let mut chain_2 = chain_1.clone();
        let entry = test_entry();
        let entry_type = test_entry_type();

        assert_eq!(
            None,
            chain_1
                .top_chain_header()
                .expect("could not get top chain header for chain 1")
        );
        assert_eq!(
            None,
            chain_2
                .top_chain_header()
                .expect("could not get top chain header for chain 2")
        );

        let chain_header = chain_2.push_entry(&entry_type, &entry).unwrap();

        assert_eq!(
            Some(chain_header.clone()),
            chain_2
                .top_chain_header()
                .expect("could not get top chain header after pushing to chain 2")
        );
        assert_eq!(
            chain_1
                .top_chain_header()
                .expect("could not get top chain header for comparing chain 1"),
            chain_2
                .top_chain_header()
                .expect("could not get top chain header when comparing chain 2")
        );
    }

    #[test]
    // test that adding something to the chain adds to the table
    fn table_put() {
        let table_actor = test_table_actor();
        let mut chain = Chain::new(table_actor.clone());

        let chain_header = chain
            .push_entry(&test_entry_type(), &test_entry())
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");

        let table_entry = table_actor
            .entry(&chain_header.entry_address())
            .expect("getting an entry from a table in a chain shouldn't fail")
            .expect("table should have entry");
        let chain_entry = chain
            .entry(&chain_header.entry_address())
            .expect("getting an entry from a chain shouldn't fail");

        assert_eq!(chain_header.entry_address(), &table_entry.address());
        assert_eq!(table_entry, chain_entry);
    }

    #[test]
    fn can_commit_entry() {
        let mut chain = test_chain();

        assert_eq!(
            None,
            chain
                .top_chain_header()
                .expect("could not get top chain header for test chain")
        );

        // chain top, chain header entry and headers should all line up after a push
        let entry_type_a = test_entry_type_a();
        let entry_a = test_entry_a();
        let chain_header_a = chain
            .push_entry(&entry_type_a, &entry_a)
            .expect("pushing a valid entry to an exclusively owned chain shouldn't fail");

        assert_eq!(
            Some(&chain_header_a),
            chain
                .top_chain_header()
                .expect("could not get top chain header as chain header a")
                .as_ref()
        );
        assert_eq!(&entry_a.address(), chain_header_a.entry_address());
        assert_eq!(&entry_a.address(), chain_header_a.entry_address());

        // we should be able to do it again
        let entry_type_b = test_entry_type_b();
        let entry_b = test_entry_b();
        let chain_header_b = chain
            .push_entry(&entry_type_b, &entry_b)
            .expect("pushing a valid entry to an exclusively owned chain shouldn't fail");

        assert_eq!(
            Some(&chain_header_b),
            chain
                .top_chain_header()
                .expect("could not get top chain_header for chain_header 2")
                .as_ref()
        );
        assert_eq!(&entry_b.address(), chain_header_b.entry_address());
        assert_eq!(&entry_b.address(), chain_header_b.entry_address());
    }

    #[test]
    /// test chain.push() and chain.get() together
    fn round_trip() {
        let mut chain = test_chain();
        let entry_type = test_entry_type();
        let entry = test_entry();
        let chain_header = chain
            .push_entry(&entry_type, &entry)
            .expect("pushing a valid entry to an exclusively owned chain shouldn't fail");

        assert_eq!(
            entry,
            chain
                .entry(&chain_header.entry_address())
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
                let chain_header = chain
                    .push_entry(&entry_type.clone(), &entry.clone())
                    .unwrap();
                assert_eq!(
                    Some(entry.clone()),
                    chain.entry(&chain_header.entry_address()),
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

        let chain_header_a = chain
            .push_entry(&entry_type_a, &entry_a)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");
        let chain_header_b = chain
            .push_entry(&entry_type_b, &entry_b)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");

        assert_eq!(
            vec![chain_header_b, chain_header_a],
            chain.iter().collect::<Vec<ChainHeader>>()
        );
    }

    #[test]
    /// test chain.iter() functional interface
    fn iter_functional() {
        let mut chain = test_chain();

        let entry_type_a = test_entry_type_a();
        let entry_type_b = test_entry_type_b();

        let entry_a = test_entry_a();
        let entry_b = test_entry_b();

        let chain_header_a = chain
            .push_entry(&entry_type_a, &entry_a)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");
        let _chain_header_b = chain
            .push_entry(&entry_type_b, &entry_b)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");
        let chain_header_c = chain
            .push_entry(&entry_type_a, &entry_a)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");

        assert_eq!(
            vec![chain_header_c, chain_header_a],
            chain
                .iter()
                .filter(|chain_header| chain_header.entry_type() == &entry_type_a)
                .collect::<Vec<ChainHeader>>()
        );
    }

    #[test]
    fn entry_advance() {
        let mut chain = test_chain();

        let entry_type_a = test_entry_type_a();
        let entry_type_b = test_entry_type_b();
        let entry_type_c = test_entry_type_a();

        let entry_a = test_entry_a();
        let entry_b = test_entry_b();
        let entry_c = test_entry_a();

        let chain_header_a = chain
            .push_entry(&entry_type_a, &entry_a)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");
        let chain_header_b = chain
            .push_entry(&entry_type_b, &entry_b)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");

        assert_eq!(
            entry_a,
            chain
                .entry(&chain_header_a.entry_address())
                .expect("getting an entry from a chain shouldn't fail"),
        );

        let chain_header_c = chain
            .push_entry(&entry_type_c, &entry_c)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");

        assert_eq!(None, chain.entry(&Address::new()));
        assert_eq!(
            entry_a,
            chain
                .entry(&chain_header_a.entry_address())
                .expect("getting an entry from a chain shouldn't fail"),
        );
        assert_eq!(
            entry_b,
            chain
                .entry(&chain_header_b.entry_address())
                .expect("getting an entry from a chain shouldn't fail"),
        );
        assert_eq!(
            entry_c,
            chain
                .entry(&chain_header_c.entry_address())
                .expect("getting an entry from a chain shouldn't fail"),
        );
    }

    #[test]
    fn entry() {
        let mut chain = test_chain();

        let entry_type_a = test_entry_type_a();
        let entry_type_b = test_entry_type_b();
        let entry_type_c = test_entry_type_a();

        let entry_a = test_entry_a();
        let entry_b = test_entry_b();
        let entry_c = test_entry_a();

        let chain_header_a = chain
            .push_entry(&entry_type_a, &entry_a)
            .expect("pushing a valid entry to an exclusively owned chain shouldn't fail");
        let chain_header_b = chain
            .push_entry(&entry_type_b, &entry_b)
            .expect("pushing a valid entry to an exclusively owned chain shouldn't fail");
        let chain_header_c = chain
            .push_entry(&entry_type_c, &entry_c)
            .expect("pushing a valid entry to an exclusively owned chain shouldn't fail");

        assert_eq!(None, chain.entry(&Address::new()));
        assert_eq!(
            entry_a,
            chain
                .entry(&chain_header_a.entry_address())
                .expect("getting an entry from a chain shouldn't fail"),
        );
        assert_eq!(
            entry_b,
            chain
                .entry(&chain_header_b.entry_address())
                .expect("getting an entry from a chain shouldn't fail"),
        );
        assert_eq!(
            entry_c,
            chain
                .entry(&chain_header_c.entry_address())
                .expect("getting an entry from a chain shouldn't fail"),
        );
    }

    #[test]
    fn top_chain_header_of_type_test() {
        let mut chain = test_chain();

        assert_eq!(None, chain.top_chain_header_of_type(&test_entry_type_a()));
        assert_eq!(None, chain.top_chain_header_of_type(&test_entry_type_b()));

        let entry_type_a = test_entry_type_a();
        let entry_type_b = test_entry_type_b();
        let entry_type_c = test_entry_type_a();

        let entry_a = test_entry_a();
        let entry_b = test_entry_b();
        let entry_c = test_entry_a();

        // type a should be chain_header_a
        // type b should be None
        let chain_header_a = chain
            .push_entry(&entry_type_a, &entry_a)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");
        assert_eq!(
            Some(&chain_header_a),
            chain.top_chain_header_of_type(&entry_type_a).as_ref()
        );
        assert_eq!(None, chain.top_chain_header_of_type(&entry_type_b));

        // type a should still be chain_header_a
        // type b should be chain_header_b
        let chain_header_b = chain
            .push_entry(&entry_type_b, &entry_b)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");
        assert_eq!(
            Some(&chain_header_a),
            chain
                .top_chain_header_of_type(&test_entry_type_a())
                .as_ref()
        );
        assert_eq!(
            Some(&chain_header_b),
            chain
                .top_chain_header_of_type(&test_entry_type_b())
                .as_ref()
        );

        // type a should be chain_header_c
        // type b should still be chain_header_b
        let chain_header_c = chain
            .push_entry(&entry_type_c, &entry_c)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");

        assert_eq!(
            Some(&chain_header_c),
            chain.top_chain_header_of_type(&entry_type_c).as_ref()
        );
        assert_eq!(
            Some(&chain_header_b),
            chain.top_chain_header_of_type(&entry_type_b).as_ref()
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

        let chain_header_a = chain
            .push_entry(&entry_type_a, &entry_a)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");
        let chain_header_b = chain
            .push_entry(&entry_type_b, &entry_b)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");
        let chain_header_c = chain
            .push_entry(&entry_type_a, &entry_a)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");

        // into_iter() returns clones of chain_headers
        assert_eq!(
            vec![chain_header_c, chain_header_b, chain_header_a],
            chain.into_iter().collect::<Vec<ChainHeader>>()
        );
    }

}
