use error::HolochainError;
use hash::HashString;
use json::{FromJson, RoundTripJson, ToJson};
use key::Key;
use multihash::Hash;
use serde_json;
use std::cmp::Ordering;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
/// Meta represents an extended form of EAV (entity-attribute-value) data
/// E = the entry key for hash table lookups
/// A = the name of the meta attribute
/// V = the value of the meta attribute
/// txn = a unique (local to the source) monotonically increasing number that can be used for
///       crdt/ordering
///       @see https://papers.radixdlt.com/tempo/#logical-clocks
/// source = the agent making the meta assertion
/// signature = the asserting agent's signature of the meta assertion
pub struct EntryMeta {
    entry_hash: HashString,
    attribute: String,
    value: String,
    // @TODO implement local transaction ordering
    // @see https://github.com/holochain/holochain-rust/issues/138
    // txn: String,
    source: String,
    // @TODO implement meta data signing
    // @see https://github.com/holochain/holochain-rust/issues/139
    // signature: String,
}

impl Ord for EntryMeta {
    fn cmp(&self, other: &EntryMeta) -> Ordering {
        // we want to sort by entry hash, then attribute name, then attribute value
        match self.entry_hash.cmp(&other.entry_hash) {
            Ordering::Equal => match self.attribute.cmp(&other.attribute) {
                Ordering::Equal => self.value.cmp(&other.value),
                Ordering::Greater => Ordering::Greater,
                Ordering::Less => Ordering::Less,
            },
            Ordering::Greater => Ordering::Greater,
            Ordering::Less => Ordering::Less,
        }
    }
}

impl PartialOrd for EntryMeta {
    fn partial_cmp(&self, other: &EntryMeta) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl EntryMeta {
    /// Builds a new Meta from EAV and agent keys, where E is an existing Entry
    /// @TODO need a `from()` to build a local meta from incoming network messages
    /// @see https://github.com/holochain/holochain-rust/issues/140
    pub fn new(node_id: &str, hash: &HashString, attribute: &str, value: &str) -> EntryMeta {
        EntryMeta {
            entry_hash: hash.clone(),
            attribute: attribute.into(),
            value: value.into(),
            source: node_id.to_string(),
        }
    }

    /// getter for entry
    pub fn entry_hash(&self) -> &HashString {
        &self.entry_hash
    }

    /// getter for attribute clone
    pub fn attribute(&self) -> String {
        self.attribute.clone()
    }

    /// getter for value clone
    pub fn value(&self) -> String {
        self.value.clone()
    }

    /// getter for source clone
    pub fn source(&self) -> String {
        self.source.clone()
    }

    pub fn make_hash(entry_hash: &HashString, attribute_name: &str) -> HashString {
        let pieces: [String; 2] = [entry_hash.clone().to_string(), attribute_name.to_string()];
        let string_to_hash = pieces.concat();

        // @TODO the hashing algo should not be hardcoded
        // @see https://github.com/holochain/holochain-rust/issues/104
        HashString::encode_from_str(&string_to_hash, Hash::SHA2256)
    }
}

impl Key for EntryMeta {
    /// the key for HashTable lookups, e.g. table.meta()
    fn key(&self) -> HashString {
        HashString::encode_from_serializable(&self, Hash::SHA2256)
    }
}

impl ToJson for EntryMeta {
    fn to_json(&self) -> Result<String, HolochainError> {
        Ok(serde_json::to_string(&self)?)
    }
}

impl FromJson for EntryMeta {
    /// @TODO accept canonical JSON
    /// @see https://github.com/holochain/holochain-rust/issues/75
    fn from_json(s: &str) -> Result<Self, HolochainError> {
        Ok(serde_json::from_str(s)?)
    }
}

impl RoundTripJson for EntryMeta {}

#[cfg(test)]
pub mod tests {

    use agent::keys::tests::test_keys;
    use hash::HashString;
    use hash_table::{
        entry::{tests::test_entry, Entry},
        entry_meta::EntryMeta,
    };
    use json::{FromJson, ToJson};
    use key::Key;
    use std::cmp::Ordering;

    /// dummy test attribute name
    pub fn test_attribute() -> String {
        "meta-attribute".into()
    }

    /// dummy test attribute name, same as test_attribute()
    pub fn test_attribute_a() -> String {
        test_attribute()
    }

    /// dummy test attribute name, differs from test_attribute()
    pub fn test_attribute_b() -> String {
        "another-attribute".into()
    }

    /// dummy test attribute value
    pub fn test_value() -> String {
        "meta value".into()
    }

    /// dummy test attribute value, same as test_value()
    pub fn test_value_a() -> String {
        test_value()
    }

    /// dummy test attribute value, differs from test_value()
    pub fn test_value_b() -> String {
        "another value".into()
    }

    pub fn test_meta_for(entry: &Entry, attribute: &str, value: &str) -> EntryMeta {
        EntryMeta::new(&test_keys().node_id(), &entry.key(), attribute, value)
    }

    /// returns dummy meta for testing
    pub fn test_meta() -> EntryMeta {
        EntryMeta::new(
            &test_keys().node_id(),
            &test_entry().key(),
            &test_attribute(),
            &test_value(),
        )
    }

    /// dummy meta, same as test_meta()
    pub fn test_meta_a() -> EntryMeta {
        test_meta()
    }

    /// returns dummy meta for testing against the same entry as test_meta_a
    pub fn test_meta_b() -> EntryMeta {
        EntryMeta::new(
            &test_keys().node_id(),
            &test_entry().key(),
            &test_attribute_b(),
            &test_value_b(),
        )
    }

    #[test]
    /// smoke test EntryMeta::new()
    fn new() {
        test_meta();
    }

    #[test]
    // test meta.entry_hash()
    fn entry_hash() {
        assert_eq!(test_meta().entry_hash(), &test_entry().key());
    }

    /// test meta.attribute()
    #[test]
    fn attribute() {
        assert_eq!(test_meta().attribute(), test_attribute());
    }

    #[test]
    /// test meta.value()
    fn value() {
        assert_eq!(test_meta().value(), test_value());
    }

    #[test]
    /// test meta.source()
    fn source() {
        assert_eq!(test_meta().source(), test_keys().node_id());
    }

    #[test]
    /// test that we can sort metas with cmp
    fn cmp() {
        // basic ordering
        let m_1ax = EntryMeta::new(
            &test_keys().node_id(),
            &HashString::from("1".to_string()),
            "a",
            "x",
        );
        let m_1ay = EntryMeta::new(
            &test_keys().node_id(),
            &HashString::from("1".to_string()),
            "a",
            "y",
        );
        let m_1bx = EntryMeta::new(
            &test_keys().node_id(),
            &HashString::from("1".to_string()),
            "b",
            "x",
        );
        let m_2ax = EntryMeta::new(
            &test_keys().node_id(),
            &HashString::from("2".to_string()),
            "a",
            "x",
        );

        // sort by entry key
        assert_eq!(Ordering::Less, m_1ax.cmp(&m_2ax));
        assert_eq!(Ordering::Equal, m_1ax.cmp(&m_1ax));
        assert_eq!(Ordering::Greater, m_2ax.cmp(&m_1ax));
        assert_eq!(Ordering::Less, m_1ay.cmp(&m_2ax));

        // entry key with operators
        assert!(m_1ax < m_2ax);
        assert!(m_2ax > m_1ax);
        assert!(m_1ay < m_2ax);

        // sort by attribute key
        assert_eq!(Ordering::Less, m_1ax.cmp(&m_1bx));
        assert_eq!(Ordering::Greater, m_1bx.cmp(&m_1ax));

        // attribute key with operators
        assert!(m_1ax < m_1bx);
        assert!(m_1bx > m_1ax);

        // sort by attribute value
        assert_eq!(Ordering::Less, m_1ax.cmp(&m_1ay));
        assert_eq!(Ordering::Greater, m_1ay.cmp(&m_1ax));

        // attribute value with operators
        assert!(m_1ax < m_1ay);
        assert!(m_1ay > m_1ax);
    }

    #[test]
    /// test the RoundTripJson implementation
    fn test_json_round_trip() {
        let meta = test_meta();
        let expected = "{\"entry_hash\":\"QmbXSE38SN3SuJDmHKSSw5qWWegvU7oTxrLDRavWjyxMrT\",\"attribute\":\"meta-attribute\",\"value\":\"meta value\",\"source\":\"test node id\"}";

        assert_eq!(expected.to_string(), meta.to_json().unwrap());
        assert_eq!(meta, EntryMeta::from_json(&expected).unwrap());
        assert_eq!(
            meta,
            EntryMeta::from_json(&meta.to_json().unwrap()).unwrap()
        );
    }
}
