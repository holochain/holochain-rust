use chain::Chain;
use hash_table::{entry::Entry, header::Header, HashTable};
use json::ToJson;
use json::FromJson;
use json::RoundTripJson;
use serde_json;
use error::HolochainError;

/// Pairs are entries with their headers
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Pair {
    header: Header,
    entry: Entry,
}

impl Pair {
    /// build a new Pair from a chain and entry
    ///
    /// Header is generated automatically
    ///
    /// a Pair is immutable, but the chain is mutable if chain.push() is used.
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
    /// @see chain::header::Header
    pub fn new<T: HashTable>(chain: &Chain<T>, entry: Entry) -> Pair {
        let header = Header::new(chain, &entry);

        let p = Pair {
            header: header,
            entry: entry,
        };

        // we panic as no code path should attempt to create invalid pairs
        // creating a Pair is an internal process of chain.push() and is deterministic based on
        // an immutable Entry (that itself cannot be invalid), so this should never happen.
        assert!(p.validate(), "attempted to create an invalid pair");

        p
    }

    /// header getter
    pub fn header(&self) -> &Header {
        &self.header
    }

    /// entry getter
    pub fn entry(&self) -> &Entry {
        &self.entry
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
}

/// @TODO return canonical JSON
/// @see https://github.com/holochain/holochain-rust/issues/75
impl ToJson for Pair {
    fn to_json(&self) -> Result<String, HolochainError> {
        // @TODO error handling
        // @see https://github.com/holochain/holochain-rust/issues/168
        let result = serde_json::to_string(&self);
        match result {
            Ok(r) => Ok(r),
            Err(e) => Err(HolochainError::SerializationError(e.to_string())),
        }
    }
}

impl FromJson for Pair {
    /// @TODO accept canonical JSON
    /// @see https://github.com/holochain/holochain-rust/issues/75
    fn from_json(s: &str) -> Result<Self, HolochainError> {
        let result: Result<Self, serde_json::Error> = serde_json::from_str(s);
        match result {
            Ok(r) => Ok(r),
            Err(e) => Err(HolochainError::SerializationError(e.to_string())),
        }
    }
}

impl RoundTripJson for Pair {}

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
    use json::ToJson;
    use json::FromJson;

    /// dummy pair
    pub fn test_pair() -> Pair {
        Pair::new(&test_chain(), test_entry())
    }

    /// dummy pair, same as test_pair()
    pub fn test_pair_a() -> Pair {
        test_pair()
    }

    /// dummy pair, differs from test_pair()
    pub fn test_pair_b() -> Pair {
        Pair::new(&test_chain(), test_entry_b())
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

        let p1 = Pair::new(&chain, e1.clone());
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
        let h = Header::new(&chain, &e);
        let p = Pair::new(&chain, e);

        assert_eq!(&h, p.header());
    }

    #[test]
    /// tests for pair.entry()
    fn entry() {
        let mut chain = test_chain();
        let t = "foo";
        let e = Entry::new(t, "");
        let p = chain
            .push(&e)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");

        assert_eq!(&e, p.entry());
    }

    #[test]
    /// tests for pair.validate()
    fn validate() {
        let chain = test_chain();
        let t = "fooType";

        let e1 = Entry::new(t, "bar");
        let p1 = Pair::new(&chain, e1);

        assert!(p1.validate());
    }

    #[test]
    /// test JSON roundtrip for pairs
    fn json_roundtrip() {
        let json = r#"{"header":{"entry_type":"testEntryType","time":"","next":null,"entry":"QmbXSE38SN3SuJDmHKSSw5qWWegvU7oTxrLDRavWjyxMrT","type_next":null,"signature":""},"entry":{"content":"test entry content","entry_type":"testEntryType"}}"#;

        assert_eq!(json, test_pair().to_json().unwrap());

        assert_eq!(test_pair(), Pair::from_json(&json).unwrap());

        assert_eq!(test_pair(), Pair::from_json(&test_pair().to_json().unwrap()).unwrap());
    }
}
