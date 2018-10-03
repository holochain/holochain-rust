use cas::content::{Address, AddressableContent, Content};
use error::HolochainError;
use hash_table::{
    entry::Entry,
    sys_entry::{EntryType, ToEntry},
};
use json::ToJson;
use serde_json;

/// Header of a source chain "Item"
/// The hash of the Header is used as the Item's key in the source chain hash table
/// Headers are linked to next header in chain and next header of same type in chain
// @TODO - serialize properties as defined in HeadersEntrySchema from golang alpha 1
// @see https://github.com/holochain/holochain-proto/blob/4d1b8c8a926e79dfe8deaa7d759f930b66a5314f/entry_headers.go#L7
// @see https://github.com/holochain/holochain-rust/issues/75
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Header {
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

impl PartialEq for Header {
    fn eq(&self, other: &Header) -> bool {
        self.address() == other.address()
    }
}

impl Header {
    /// build a new Header from a chain, entry type and entry.
    /// a Header is immutable, but the chain is mutable if chain.push() is used.
    /// this means that a header becomes invalid and useless as soon as the chain is mutated
    /// the only valid usage of a header is to immediately push it onto a chain in a Pair.
    /// normally (outside unit tests) the generation of valid headers is internal to the
    /// chain::SourceChain trait and should not need to be handled manually
    ///
    /// @see chain::pair::Pair
    /// @see chain::entry::Entry
    pub fn new(
        entry_type: &EntryType,
        timestamp: &str,
        link: Option<Address>,
        entry_address: &Address,
        entry_signature: &str,
        link_same_type: Option<Address>,
    ) -> Self {
        Header {
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

    /// returns true if the entry type is a system entry
    pub fn is_sys(&self) -> bool {
        match self.entry_type {
            EntryType::App(_) => true,
            _ => false,
        }
    }

    /// returns true if the entry type is an app entry
    pub fn is_app(&self) -> bool {
        !self.is_sys()
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

impl ToJson for Header {
    fn to_json(&self) -> Result<String, HolochainError> {
        Ok(serde_json::to_string(self)?)
    }
}

//
impl ToEntry for Header {
    fn to_entry(&self) -> (EntryType, Entry) {
        (
            EntryType::Header,
            Entry::from(self.to_json().expect("entry should be valid")),
        )
    }

    fn from_entry(entry: &Entry) -> Self {
        return Header::from_json_str(&entry.content()).expect("entry is not a valid Header Entry");
    }
}

impl AddressableContent for Header {
    fn content(&self) -> Content {
        self.to_json().expect("could not Jsonify Header as Content")
    }

    fn from_content(content: &Content) -> Self {
        Header::from_json_str(content).expect("could not read Json as valid Header Content")
    }
}

#[cfg(test)]
pub mod tests {
    use cas::content::{Address, AddressableContent};
    use chain::{header::Header, pair::tests::test_pair, tests::test_chain, SourceChain};
    use hash::HashString;
    use hash_table::{
        entry::tests::{
            test_entry, test_entry_a, test_entry_b, test_entry_type, test_entry_type_a,
            test_entry_type_b,
        },
        sys_entry::ToEntry,
    };

    /// returns a dummy header for use in tests
    pub fn test_header() -> Header {
        test_pair().header().clone()
    }

    pub fn test_header_address() -> Address {
        Address::from("Qmc1n5gbUU2QKW6is9ENTqmaTcEjYMBwNkcACCxe3bBDnd".to_string())
    }

    #[test]
    /// tests for PartialEq
    fn eq() {
        let chain_a = test_chain();

        let entry_type_a = test_entry_type_a();
        let entry_type_b = test_entry_type_b();

        let entry_a = test_entry_a();
        let entry_b = test_entry_b();

        // same content + chain state is equal
        assert_eq!(
            chain_a.create_next_header(&entry_type_a, &entry_a),
            chain_a.create_next_header(&entry_type_a, &entry_a),
        );

        // different content is different
        assert_ne!(
            chain_a.create_next_header(&entry_type_a, &entry_a),
            chain_a.create_next_header(&entry_type_a, &entry_b),
        );

        // different type is different
        assert_ne!(
            chain_a.create_next_header(&entry_type_a, &entry_a),
            chain_a.create_next_header(&entry_type_b, &entry_a),
        );

        // different state is different with same entry
        let mut chain_b = test_chain();
        chain_b
            .push_entry(&entry_type_a, &entry_a)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");

        assert_ne!(
            chain_a.create_next_header(&entry_type_a, &entry_a),
            chain_b.create_next_header(&entry_type_a, &entry_a)
        );
    }

    #[test]
    /// tests for Header::new()
    fn new() {
        let chain = test_chain();
        let entry_type = test_entry_type();
        let entry = test_entry();

        let header = chain.create_next_header(&entry_type, &entry);

        assert_eq!(header.entry_address(), &entry.address());
        assert_eq!(header.link(), None);
        assert_ne!(header.address(), HashString::new());
    }

    #[test]
    /// tests for header.entry_type()
    fn entry_type() {
        let chain = test_chain();
        let entry_type = test_entry_type();
        let entry = test_entry();

        let header = chain.create_next_header(&entry_type, &entry);

        assert_eq!(header.entry_type(), &entry_type);
    }

    #[test]
    /// tests for header.time()
    fn time() {
        let chain = test_chain();
        let entry_type = test_entry_type();
        let entry = test_entry();

        let header = chain.create_next_header(&entry_type, &entry);

        assert_eq!(header.timestamp(), "");
    }

    #[test]
    /// tests for header.next()
    fn next() {
        let mut chain = test_chain();

        let entry_type_a = test_entry_type_a();
        let entry_type_b = test_entry_type_b();

        let entry_a = test_entry_a();
        let entry_b = test_entry_b();

        // first header is genesis so next should be None
        let pair_a = chain
            .push_entry(&entry_type_a, &entry_a)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");
        let header_a = pair_a.header();

        assert_eq!(header_a.link(), None);

        // second header next should be first header hash
        let pair_b = chain
            .push_entry(&entry_type_b, &entry_b)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");
        let header_b = pair_b.header();

        assert_eq!(header_b.link(), Some(header_a.to_entry().1.address()));
    }

    #[test]
    /// tests for header.entry()
    fn entry() {
        let chain = test_chain();
        let entry_type = test_entry_type();
        let entry = test_entry();

        // header for an entry should contain the entry hash under entry()
        let header = chain.create_next_header(&entry_type, &entry);

        assert_eq!(header.entry_address(), &entry.address());
    }

    #[test]
    /// tests for header.type_next()
    fn type_next() {
        let mut chain = test_chain();

        let entry_type_a = test_entry_type_a();
        let entry_type_b = test_entry_type_b();

        let entry_a = test_entry_a();
        let entry_b = test_entry_b();

        // first header is genesis so next should be None
        let pair_a = chain
            .push_entry(&entry_type_a, &entry_a)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");
        let header_a = pair_a.header();

        assert_eq!(header_a.link_same_type(), None);

        // second header is a different type so next should be None
        let pair_b = chain
            .push_entry(&entry_type_b, &entry_b)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");
        let header_b = pair_b.header();

        assert_eq!(header_b.link_same_type(), None);

        // third header is same type as first header so next should be first header hash
        let pair_c = chain
            .push_entry(&entry_type_a, &entry_b)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");
        let header_c = pair_c.header();

        assert_eq!(header_c.link_same_type(), Some(header_a.address()));
    }

    #[test]
    /// tests for header.signature()
    fn signature() {
        let chain = test_chain();
        let entry_type = test_entry_type();
        let entry = test_entry();

        let header = chain.create_next_header(&entry_type, &entry);

        assert_eq!("", header.entry_signature());
    }

    #[test]
    /// test header.address() against a known value
    fn known_address() {
        let chain = test_chain();
        let entry_type = test_entry_type();
        let entry = test_entry();

        // check a known hash
        let header = chain.create_next_header(&entry_type, &entry);

        assert_eq!(test_header_address(), header.address());
    }

    #[test]
    /// test that different entry content returns different addresses
    fn address_entry_content() {
        let chain = test_chain();

        let entry_type_a = test_entry_type_a();
        let entry_type_b = test_entry_type_b();

        let entry_a = test_entry_a();
        let entry_b = test_entry_b();

        // different entries must return different hashes
        let header_a = chain.create_next_header(&entry_type_a, &entry_a);

        let header_b = chain.create_next_header(&entry_type_b, &entry_b);

        assert_ne!(header_a.address(), header_b.address());

        let entry_type_c = test_entry_type_a();
        let entry_c = test_entry_a();

        // same entry must return same address
        let header_c = chain.create_next_header(&entry_type_c, &entry_c);

        assert_eq!(header_a.address(), header_c.address());
    }

    #[test]
    /// test that different entry types returns different addresses
    fn address_entry_type() {
        let chain = test_chain();

        let entry_type_a = test_entry_type_a();
        let entry_type_b = test_entry_type_b();

        let entry = test_entry();

        let header_a = chain.create_next_header(&entry_type_a, &entry);
        let header_b = chain.create_next_header(&entry_type_b, &entry);

        // different types must give different addresses
        assert_ne!(header_a.address(), header_b.address());
    }

    #[test]
    /// test that different chain state returns different addresses
    fn address_chain_state() {
        // different chain, different address
        let mut chain = test_chain();

        let entry_type = test_entry_type();
        let entry = test_entry();

        let header = chain.create_next_header(&entry_type, &entry);

        let pair_a = chain
            .push_entry(&entry_type, &entry)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");
        // p2 will have a different address to p1 with the same entry as the chain state is different
        let pair_b = chain
            .push_entry(&entry_type, &entry)
            .expect("pushing a valid entry to an exlusively owned chain shouldn't fail");

        assert_eq!(header.address(), pair_a.header().address());
        assert_ne!(header.address(), pair_b.header().address());
    }

    #[test]
    /// test that different type_next returns different addresses
    fn address_type_next() {
        // @TODO is it possible to test that type_next changes the address in an isolated way?
        // @see https://github.com/holochain/holochain-rust/issues/76
    }

    /// Committing a LinkEntry to source chain should work
    #[test]
    fn can_round_trip_header_entry() {
        let chain = test_chain();
        let entry_type = test_entry_type();
        let entry = test_entry();

        let header = chain.create_next_header(&entry_type, &entry);

        let header_entry = header.to_entry().1;
        let header_trip = Header::from_entry(&header_entry);

        assert_eq!(header, header_trip);
    }
}
