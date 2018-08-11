use chain::Chain;
use hash_table::{entry::Entry, header::Header};
use serde_json;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Pair {
    header: Header,
    entry: Entry,
}

impl Pair {
    /// build a new Pair from a chain and entry
    /// Header is generated automatically
    /// a Pair is immutable, but the chain is mutable if chain.push() is used.
    /// this means that if two Pairs X and Y are generated for chain C then Pair X is pushed onto
    /// C to create chain C' (containing X), then Pair Y is no longer valid as the headers would
    /// need to include X. Pair Y can be regenerated with the same parameters as Y' and will be
    /// now be valid, the new Y' will include correct headers pointing to X.
    /// @see chain::entry::Entry
    /// @see chain::header::Header
    pub fn new(chain: &Chain, entry: &Entry) -> Pair {
        let header = Header::new(chain, entry);

        let p = Pair {
            header: header.clone(),
            entry: entry.clone(),
        };

        if !p.validate() {
            // we panic as no code path should attempt to create invalid pairs
            // creating a Pair is an internal process of chain.push() and is deterministic based on
            // an immutable Entry (that itself cannot be invalid), so this should never happen.
            panic!("attempted to create an invalid pair");
        };

        p
    }

    /// header getter
    pub fn header(&self) -> Header {
        self.header.clone()
    }

    /// entry getter
    pub fn entry(&self) -> Entry {
        self.entry.clone()
    }

    /// key used in hash table lookups and other references
    pub fn key(&self) -> String {
        self.header.hash()
    }

    /// true if the pair is valid
    pub fn validate(&self) -> bool {
        // the header and entry must validate independently
        self.header.validate() && self.entry.validate()
        // the header entry hash must be the same as the entry hash
        && self.header.entry() == self.entry.hash()
        // the entry_type must line up across header and entry
        && self.header.entry_type() == self.entry.entry_type()
    }

    /// serialize the Pair to a canonical JSON string
    /// @TODO return canonical JSON
    /// @see https://github.com/holochain/holochain-rust/issues/75
    pub fn to_json(&self) -> String {
        // @TODO error handling
        // @see https://github.com/holochain/holochain-rust/issues/168
        serde_json::to_string(&self).unwrap()
    }

    /// deserialize a Pair from a canonical JSON string
    /// @TODO accept canonical JSON
    /// @see https://github.com/holochain/holochain-rust/issues/75
    pub fn from_json(s: &str) -> Pair {
        let pair: Pair = serde_json::from_str(s).unwrap();
        pair
    }
}

#[cfg(test)]
pub mod tests {
    use super::Pair;
    use chain::tests::test_chain;
    use hash_table::{
        entry::{
            tests::{test_entry, test_entry_b},
            Entry,
        },
        header::Header,
    };

    /// dummy pair
    pub fn test_pair() -> Pair {
        Pair::new(&test_chain(), &test_entry())
    }

    /// dummy pair, same as test_pair()
    pub fn test_pair_a() -> Pair {
        test_pair()
    }

    /// dummy pair, differs from test_pair()
    pub fn test_pair_b() -> Pair {
        Pair::new(&test_chain(), &test_entry_b())
    }

    #[test]
    /// tests for Pair::new()
    fn new() {
        let chain = test_chain();
        let t = "fooType";
        let e1 = Entry::new(t, "some content");
        let h1 = Header::new(&chain, &e1);

        assert_eq!(h1.entry(), e1.hash());
        assert_eq!(h1.next(), None);

        let p1 = Pair::new(&chain, &e1);
        assert_eq!(e1, p1.entry());
        assert_eq!(h1, p1.header());
    }

    #[test]
    /// tests for pair.header()
    fn header() {
        let chain = test_chain();
        let t = "foo";
        let c = "bar";
        let e = Entry::new(t, c);
        let h = Header::new(&chain, &e);
        let p = Pair::new(&chain, &e);

        assert_eq!(h, p.header());
    }

    #[test]
    /// tests for pair.entry()
    fn entry() {
        let mut chain = test_chain();
        let t = "foo";
        let e = Entry::new(t, "");
        let p = chain.push(&e).unwrap();

        assert_eq!(e, p.entry());
    }

    #[test]
    /// tests for pair.validate()
    fn validate() {
        let chain = test_chain();
        let t = "fooType";

        let e1 = Entry::new(t, "bar");
        let p1 = Pair::new(&chain, &e1);

        assert!(p1.validate());
    }

    #[test]
    /// test JSON roundtrip for pairs
    fn json_roundtrip() {
        let json = "{\"header\":{\"entry_type\":\"testEntryType\",\"time\":\"\",\"next\":null,\"entry\":\"QmbXSE38SN3SuJDmHKSSw5qWWegvU7oTxrLDRavWjyxMrT\",\"type_next\":null,\"signature\":\"\"},\"entry\":{\"content\":\"test entry content\",\"entry_type\":\"testEntryType\"}}";

        assert_eq!(json, test_pair().to_json(),);

        assert_eq!(test_pair(), Pair::from_json(&json),);

        assert_eq!(test_pair(), Pair::from_json(&test_pair().to_json()),);
    }
}
