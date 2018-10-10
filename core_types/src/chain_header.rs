use cas::content::{Address, AddressableContent, Content};
use entry::{test_entry, Entry};
use entry_type::{test_entry_type, EntryType};
use error::HolochainError;
use json::ToJson;
use serde_json;
use to_entry::ToEntry;

/// ChainHeader of a source chain "Item"
/// The hash of the ChainHeader is used as the Item's key in the source chain hash table
/// ChainHeaders are linked to next header in chain and next header of same type in chain
// @TODO - serialize properties as defined in ChainHeadersEntrySchema from golang alpha 1
// @see https://github.com/holochain/holochain-proto/blob/4d1b8c8a926e79dfe8deaa7d759f930b66a5314f/entry_headers.go#L7
// @see https://github.com/holochain/holochain-rust/issues/75
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChainHeader {
    /// the type of this entry
    /// system types may have associated "subconscious" behavior
    entry_type: EntryType,
    /// ISO8601 time stamp
    timestamp: String,
    /// Key to the immediately preceding header. Only the genesis Pair can have None as valid
    link: Option<Address>,
    /// Key to the entry of this header
    entry_address: Address,
    /// agent's cryptographic signature of the entry
    entry_signature: String,
    /// Key to the most recent header of the same type, None is valid only for the first of that type
    link_same_type: Option<Address>,
}

impl PartialEq for ChainHeader {
    fn eq(&self, other: &ChainHeader) -> bool {
        self.address() == other.address()
    }
}

impl ChainHeader {
    /// build a new ChainHeader from a chain, entry type and entry.
    /// a ChainHeader is immutable, but the chain is mutable if chain.push() is used.
    /// this means that a header becomes invalid and useless as soon as the chain is mutated
    /// the only valid usage of a header is to immediately push it onto a chain in a Pair.
    /// normally (outside unit tests) the generation of valid headers is internal to the
    /// chain::SourceChain trait and should not need to be handled manually
    ///
    /// @see chain::entry::Entry
    pub fn new(
        entry_type: &EntryType,
        timestamp: &str,
        link: Option<Address>,
        entry_address: &Address,
        entry_signature: &str,
        link_same_type: Option<Address>,
    ) -> Self {
        ChainHeader {
            entry_type: entry_type.to_owned(),
            timestamp: timestamp.to_string(),
            link: link,
            entry_address: entry_address.clone(),
            entry_signature: entry_signature.to_string(),
            link_same_type: link_same_type,
        }
    }

    pub fn from_json_str(header_str: &str) -> serde_json::Result<Self> {
        serde_json::from_str(header_str)
    }

    /// entry_type getter
    pub fn entry_type(&self) -> &EntryType {
        &self.entry_type
    }

    /// timestamp getter
    pub fn timestamp(&self) -> &str {
        &self.timestamp
    }

    /// link getter
    pub fn link(&self) -> Option<Address> {
        self.link.clone()
    }

    /// entry_address getter
    pub fn entry_address(&self) -> &Address {
        &self.entry_address
    }

    /// link_same_type getter
    pub fn link_same_type(&self) -> Option<Address> {
        self.link_same_type.clone()
    }

    /// entry_signature getter
    pub fn entry_signature(&self) -> &str {
        &self.entry_signature
    }
}

impl ToJson for ChainHeader {
    fn to_json(&self) -> Result<String, HolochainError> {
        Ok(serde_json::to_string(self)?)
    }
}

//
impl ToEntry for ChainHeader {
    fn to_entry(&self) -> (EntryType, Entry) {
        (
            EntryType::ChainHeader,
            Entry::from(self.to_json().expect("entry should be valid")),
        )
    }

    fn from_entry(entry: &Entry) -> Self {
        return ChainHeader::from_json_str(&entry.content())
            .expect("entry is not a valid ChainHeader Entry");
    }
}

impl AddressableContent for ChainHeader {
    fn content(&self) -> Content {
        self.to_json()
            .expect("could not Jsonify ChainHeader as Content")
    }

    fn from_content(content: &Content) -> Self {
        ChainHeader::from_json_str(content)
            .expect("could not read Json as valid ChainHeader Content")
    }
}

/// returns a dummy header for use in tests
pub fn test_chain_header() -> ChainHeader {
    ChainHeader::new(
        &test_entry_type(),
        &String::new(),
        None,
        &test_entry().address(),
        &String::new(),
        None,
    )
}

#[cfg(test)]
pub mod tests {
    use cas::content::{Address, AddressableContent};
    use chain_header::{test_chain_header, ChainHeader};
    use entry::{test_entry, test_entry_b};
    use entry_type::{test_entry_type, test_entry_type_a, test_entry_type_b};
    use to_entry::ToEntry;

    /// returns a dummy header for use in tests
    pub fn test_chain_header_a() -> ChainHeader {
        test_chain_header()
    }

    /// returns a dummy header for use in tests. different from test_chain_header_a.
    pub fn test_chain_header_b() -> ChainHeader {
        ChainHeader::new(
            &test_entry_type_b(),
            &String::new(),
            None,
            &test_entry_b().address(),
            &String::new(),
            None,
        )
    }

    pub fn test_header_address() -> Address {
        Address::from("Qmc1n5gbUU2QKW6is9ENTqmaTcEjYMBwNkcACCxe3bBDnd".to_string())
    }

    #[test]
    /// tests for PartialEq
    fn eq() {
        // basic equality
        assert_eq!(test_chain_header(), test_chain_header());

        // different content is different
        assert_ne!(test_chain_header_a(), test_chain_header_b());

        // different type is different
        assert_ne!(
            ChainHeader::new(
                &test_entry_type_a(),
                &String::new(),
                None,
                &test_entry().address(),
                &String::new(),
                None
            ),
            ChainHeader::new(
                &test_entry_type_b(),
                &String::new(),
                None,
                &test_entry().address(),
                &String::new(),
                None
            ),
        );

        // different previous header is different
        assert_ne!(
            ChainHeader::new(
                &test_entry_type(),
                &String::new(),
                None,
                &test_entry().address(),
                &String::new(),
                None
            ),
            ChainHeader::new(
                &test_entry_type(),
                &String::new(),
                Some(test_chain_header().address()),
                &test_entry().address(),
                &String::new(),
                None
            ),
        );
    }

    #[test]
    /// tests for ChainHeader::new()
    fn new() {
        let chain_header = test_chain_header();

        assert_eq!(chain_header.entry_address(), &test_entry().address());
        assert_eq!(chain_header.link(), None);
        assert_ne!(chain_header.address(), Address::new());
    }

    #[test]
    /// tests for header.entry_type()
    fn entry_type() {
        assert_eq!(test_chain_header().entry_type(), &test_entry_type());
    }

    #[test]
    /// tests for header.time()
    fn time() {
        assert_eq!(test_chain_header().timestamp(), "");
    }

    #[test]
    fn link_test() {
        let chain_header_a = test_chain_header();
        let chain_header_b = ChainHeader::new(
            &test_entry_type(),
            &String::new(),
            Some(chain_header_a.address()),
            &test_entry().address(),
            &String::new(),
            None,
        );
        assert_eq!(None, chain_header_a.link());
        assert_eq!(Some(chain_header_a.address()), chain_header_b.link());
    }

    #[test]
    fn entry_test() {
        assert_eq!(test_chain_header().entry_address(), &test_entry().address());
    }

    #[test]
    fn link_same_type_test() {
        let chain_header_a = test_chain_header();
        let chain_header_b = ChainHeader::new(
            &test_entry_type_b(),
            &String::new(),
            Some(chain_header_a.address()),
            &test_entry().address(),
            &String::new(),
            None,
        );
        let chain_header_c = ChainHeader::new(
            &test_entry_type(),
            &String::new(),
            Some(chain_header_b.address()),
            &test_entry().address(),
            &String::new(),
            Some(chain_header_a.address()),
        );

        assert_eq!(None, chain_header_a.link_same_type());
        assert_eq!(None, chain_header_b.link_same_type());
        assert_eq!(
            Some(chain_header_a.address()),
            chain_header_c.link_same_type()
        );
    }

    #[test]
    /// tests for chain_header.signature()
    fn signature() {
        assert_eq!("", test_chain_header().entry_signature());
    }

    #[test]
    /// test header.address() against a known value
    fn known_address() {
        assert_eq!(
            test_chain_header_a().address(),
            test_chain_header().address()
        );
    }

    #[test]
    /// test that different entry content returns different addresses
    fn address_entry_content() {
        assert_ne!(
            test_chain_header_a().address(),
            test_chain_header_b().address()
        );
    }

    #[test]
    /// test that different entry types returns different addresses
    fn address_entry_type() {
        assert_ne!(
            ChainHeader::new(
                &test_entry_type_a(),
                &String::new(),
                None,
                &test_entry().address(),
                &String::new(),
                None
            ).address(),
            ChainHeader::new(
                &test_entry_type_b(),
                &String::new(),
                None,
                &test_entry().address(),
                &String::new(),
                None
            ).address(),
        );
    }

    #[test]
    /// test that different chain state returns different addresses
    fn address_chain_state() {
        assert_ne!(
            test_chain_header().address(),
            ChainHeader::new(
                &test_entry_type_a(),
                &String::new(),
                Some(test_chain_header().address()),
                &test_entry().address(),
                &String::new(),
                None
            ).address(),
        );
    }

    #[test]
    /// test that different type_next returns different addresses
    fn address_type_next() {
        assert_ne!(
            test_chain_header().address(),
            ChainHeader::new(
                &test_entry_type_a(),
                &String::new(),
                None,
                &test_entry().address(),
                &String::new(),
                Some(test_chain_header().address())
            ).address(),
        );
    }

    /// Committing a LinkEntry to source chain should work
    #[test]
    fn can_round_trip_header_entry() {
        assert_eq!(
            test_chain_header(),
            ChainHeader::from_entry(&test_chain_header().to_entry().1)
        );
    }
}
