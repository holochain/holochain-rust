use actor::Protocol;
use chain::header::Header;
use error::HolochainError;
use hash::HashString;
use hash_table::{entry::Entry, sys_entry::ToEntry, HashTable};
use json::{FromJson, RoundTripJson, ToJson};
use key::Key;
use riker::actors::*;
use serde_json;

/// Struct for holding a source chain "Item"
/// It is like a pair holding the entry and header separately
/// The source chain being a hash table, the key of a Pair is the hash of its Header
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Pair {
    header: Header,
    entry: Entry,
}

impl Pair {
    /// Reconstruct Pair from Header stored in a HashTable
    pub fn from_header(table: &ActorRef<Protocol>, header: &Header) -> Option<Self> {
        let entry = table
            .entry(&header.entry_hash())
            .expect("should not attempt to create invalid pair");
        if entry.is_none() {
            return None;
        }

        Some(Pair {
            header: header.clone(),
            entry: entry.expect("should not attempt to create invalid pair"),
        })
    }

    /// Standard constructor
    pub fn new(header: &Header, entry: &Entry) -> Self {
        Pair {
            header: header.clone(),
            entry: entry.clone(),
        }
    }

    /// header getter
    pub fn header(&self) -> &Header {
        &self.header
    }

    /// entry getter
    pub fn entry(&self) -> &Entry {
        &self.entry
    }

    /// Return true if the pair is valid
    pub fn validate(&self) -> bool {
        // the header and entry must validate independently
        self.header.validate() && self.entry.validate()
        // the header entry hash must be the same as the entry hash
        && self.header.entry_hash() == &self.entry.hash()
        // the entry_type must line up across header and entry
        && self.header.entry_type() == self.entry.entry_type()
    }
}

impl Key for Pair {
    fn key(&self) -> HashString {
        //        self.header.hash()
        self.header.to_entry().key()
    }
}

/// @TODO return canonical JSON
/// @see https://github.com/holochain/holochain-rust/issues/75
impl ToJson for Pair {
    fn to_json(&self) -> Result<String, HolochainError> {
        Ok(serde_json::to_string(&self)?)
    }
}

impl ToJson for Option<Pair> {
    fn to_json(&self) -> Result<String, HolochainError> {
        match self {
            Some(pair) => pair.to_json(),
            None => Ok(String::new()),
        }
    }
}

impl FromJson for Pair {
    /// @TODO accept canonical JSON
    /// @see https://github.com/holochain/holochain-rust/issues/75
    fn from_json(s: &str) -> Result<Self, HolochainError> {
        Ok(serde_json::from_str(s)?)
    }
}

impl RoundTripJson for Pair {}

#[cfg(test)]
pub mod tests {
    use super::Pair;
    use chain::{tests::test_chain, SourceChain};
    use hash_table::entry::{
        tests::{test_entry, test_entry_b, test_entry_unique},
        Entry,
    };
    use json::{FromJson, ToJson};

    /// dummy pair
    pub fn test_pair() -> Pair {
        test_chain().create_next_pair(&test_entry())
    }

    /// dummy pair, same as test_pair()
    pub fn test_pair_a() -> Pair {
        test_pair()
    }

    /// dummy pair, differs from test_pair()
    pub fn test_pair_b() -> Pair {
        test_chain().create_next_pair(&test_entry_b())
    }

    /// dummy pair, uses test_entry_unique()
    pub fn test_pair_unique() -> Pair {
        Pair::new(test_pair().header(), &test_entry_unique())
    }

    #[test]
    /// tests for Pair::new()
    fn new() {
        let chain = test_chain();
        let t = "fooType";
        let e1 = Entry::new(t, "some content");
        let h1 = chain.create_next_header(&e1);

        assert_eq!(h1.entry_hash(), &e1.hash());
        assert_eq!(h1.link(), None);

        let p1 = chain.create_next_pair(&e1.clone());
        assert_eq!(&e1, p1.entry());
        assert_eq!(&h1, p1.header());
    }

    #[test]
    /// tests for pair.header()
    fn header() {
        let chain = test_chain();
        let t = "foo";
        let c = "bar";
        let e = Entry::new(t, c);
        let h = chain.create_next_header(&e);
        let p = chain.create_next_pair(&e);

        assert_eq!(&h, p.header());
    }

    #[test]
    /// tests for pair.entry()
    fn entry() {
        let mut chain = test_chain();
        let t = "foo";
        let e = Entry::new(t, "");
        let p = chain
            .push_entry(&e)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");

        assert_eq!(&e, p.entry());
    }

    #[test]
    /// tests for pair.validate()
    fn validate() {
        let chain = test_chain();
        let t = "fooType";

        let e1 = Entry::new(t, "bar");
        let p1 = chain.create_next_pair(&e1);

        assert!(p1.validate());
    }

    #[test]
    /// test JSON roundtrip for pairs
    fn json_roundtrip() {
        let json = "{\"header\":{\"entry_type\":\"testEntryType\",\"timestamp\":\"\",\"link\":null,\"entry_hash\":\"QmbXSE38SN3SuJDmHKSSw5qWWegvU7oTxrLDRavWjyxMrT\",\"entry_signature\":\"\",\"link_same_type\":null},\"entry\":{\"content\":\"test entry content\",\"entry_type\":\"testEntryType\"}}"
        ;

        assert_eq!(json, test_pair().to_json().unwrap());

        assert_eq!(test_pair(), Pair::from_json(&json).unwrap());

        assert_eq!(
            test_pair(),
            Pair::from_json(&test_pair().to_json().unwrap()).unwrap()
        );
    }
}
