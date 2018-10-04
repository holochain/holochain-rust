use actor::Protocol;
use cas::content::{AddressableContent, Content};
use chain::header::ChainHeader;
use error::HolochainError;
use hash_table::{entry::Entry, HashTable};
use json::{FromJson, RoundTripJson, ToJson};
use riker::actors::*;
use serde_json;

/// Struct for holding a source chain "Item"
/// It is like a pair holding the entry and header separately
/// The source chain being a hash table, the key of a Pair is the hash of its ChainHeader
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Pair {
    header: ChainHeader,
    entry: Entry,
}

impl Pair {
    /// Reconstruct Pair from ChainHeader stored in a HashTable
    pub fn from_header(table: &ActorRef<Protocol>, header: &ChainHeader) -> Option<Self> {
        let entry = table
            .entry(&header.entry_address())
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
    pub fn new(header: &ChainHeader, entry: &Entry) -> Self {
        Pair {
            header: header.clone(),
            entry: entry.clone(),
        }
    }

    /// header getter
    pub fn header(&self) -> &ChainHeader {
        &self.header
    }

    /// entry getter
    pub fn entry(&self) -> &Entry {
        &self.entry
    }

    /// Return true if the pair is valid
    pub fn validate(&self) -> bool {
        // the header entry hash must be the same as the entry hash
        self.header().entry_address() == &self.entry().address()
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

impl AddressableContent for Pair {
    fn content(&self) -> Content {
        self.to_json().expect("could not Jsonify Pair as Content")
    }

    fn from_content(content: &Content) -> Self {
        Pair::from_json(content).expect("could not parse JSON as Pair Content")
    }
}

impl RoundTripJson for Pair {}

#[cfg(test)]
pub mod tests {
    use super::Pair;
    use cas::content::AddressableContent;
    use chain::{tests::test_chain, SourceChain};
    use hash_table::entry::tests::{
        test_entry, test_entry_b, test_entry_type, test_entry_type_b, test_entry_unique,
    };
    use json::{FromJson, ToJson};

    /// dummy pair
    pub fn test_pair() -> Pair {
        test_chain().create_next_pair(&test_entry_type(), &test_entry())
    }

    /// dummy pair, same as test_pair()
    pub fn test_pair_a() -> Pair {
        test_pair()
    }

    /// dummy pair, differs from test_pair()
    pub fn test_pair_b() -> Pair {
        test_chain().create_next_pair(&test_entry_type_b(), &test_entry_b())
    }

    /// dummy pair, uses test_entry_unique()
    pub fn test_pair_unique() -> Pair {
        Pair::new(test_pair().header(), &test_entry_unique())
    }

    #[test]
    /// tests for Pair::new()
    fn new() {
        let chain = test_chain();

        let entry_type = test_entry_type();
        let entry = test_entry();

        let header = chain.create_next_header(&entry_type, &entry);

        assert_eq!(header.entry_address(), &entry.address());
        assert_eq!(header.link(), None);

        let pair = chain.create_next_pair(&entry_type, &entry);
        assert_eq!(&entry, pair.entry());
        assert_eq!(&header, pair.header());
    }

    #[test]
    /// tests for pair.header()
    fn header() {
        let chain = test_chain();
        let entry_type = test_entry_type();
        let entry = test_entry();
        let header = chain.create_next_header(&entry_type, &entry);
        let pair = chain.create_next_pair(&entry_type, &entry);

        assert_eq!(&header, pair.header());
    }

    #[test]
    /// tests for pair.entry()
    fn entry() {
        let mut chain = test_chain();
        let entry_type = test_entry_type();
        let entry = test_entry();
        let pair = chain
            .push_entry(&entry_type, &entry)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");

        assert_eq!(&entry, pair.entry());
    }

    #[test]
    /// tests for pair.validate()
    fn validate() {
        let chain = test_chain();
        let entry_type = test_entry_type();
        let entry = test_entry();

        let pair = chain.create_next_pair(&entry_type, &entry);

        assert!(pair.validate());
    }

    #[test]
    /// test JSON roundtrip for pairs
    fn json_roundtrip() {
        let json = "{\"header\":{\"entry_type\":{\"App\":\"testEntryType\"},\"timestamp\":\"\",\"link\":null,\"entry_address\":\"QmbXSE38SN3SuJDmHKSSw5qWWegvU7oTxrLDRavWjyxMrT\",\"entry_signature\":\"\",\"link_same_type\":null},\"entry\":\"test entry content\"}"
        ;

        assert_eq!(json, test_pair().to_json().unwrap());

        assert_eq!(test_pair(), Pair::from_json(&json).unwrap());

        assert_eq!(
            test_pair(),
            Pair::from_json(&test_pair().to_json().unwrap()).unwrap()
        );
    }
}
