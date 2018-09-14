use agent::keys::Keys;
use error::HolochainError;
use hash_table::pair::Pair;
use json::{FromJson, RoundTripJson, ToJson};
use key::Key;
use multihash::Hash;
use serde_json;
use std::cmp::Ordering;
use hash::HashString;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
/// PairMeta represents an extended form of EAV (entity-attribute-value) data
/// E = the pair key for hash table lookups
/// A = the name of the meta attribute
/// V = the value of the meta attribute
/// txn = a unique (local to the source) monotonically increasing number that can be used for
///       crdt/ordering
///       @see https://papers.radixdlt.com/tempo/#logical-clocks
/// source = the agent making the meta assertion
/// signature = the asserting agent's signature of the meta assertion
pub struct PairMeta {
    pair_hash: HashString,
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

impl Ord for PairMeta {
    fn cmp(&self, other: &PairMeta) -> Ordering {
        // we want to sort by pair hash, then attribute name, then attribute value
        match self.pair_hash.cmp(&other.pair_hash) {
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

impl PartialOrd for PairMeta {
    fn partial_cmp(&self, other: &PairMeta) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl PairMeta {
    /// Builds a new PairMeta from EAV and agent keys, where E is an existing Pair
    /// @TODO need a `from()` to build a local meta from incoming network messages
    /// @see https://github.com/holochain/holochain-rust/issues/140
    pub fn new(keys: &Keys, pair: &Pair, attribute: &str, value: &str) -> PairMeta {
        PairMeta {
            pair_hash: pair.key(),
            attribute: attribute.into(),
            value: value.into(),
            source: keys.node_id(),
        }
    }

    /// getter for pair clone
    pub fn pair_hash(&self) -> HashString {
        self.pair_hash.clone()
    }

    /// getter for attribute clone
    pub fn attribute(&self) -> String {
        self.attribute.clone()
    }

    /// getter for value clone
    pub fn value(&self) -> String {
        self.value.clone()
    }

    // getter for source clone
    pub fn source(&self) -> String {
        self.source.clone()
    }
}

impl Key for PairMeta {
    fn key(&self) -> HashString {
        HashString::encode_from_serializable(&self, Hash::SHA2256)
    }
}

impl ToJson for PairMeta {
    fn to_json(&self) -> Result<String, HolochainError> {
        Ok(serde_json::to_string(&self)?)
    }
}

impl FromJson for PairMeta {
    /// @TODO accept canonical JSON
    /// @see https://github.com/holochain/holochain-rust/issues/75
    fn from_json(s: &str) -> Result<Self, HolochainError> {
        Ok(serde_json::from_str(s)?)
    }
}

impl RoundTripJson for PairMeta {}

#[cfg(test)]
pub mod tests {

    use agent::keys::tests::test_keys;
    use hash_table::{
        pair::{
            tests::{test_pair, test_pair_a, test_pair_b},
            Pair,
        },
        pair_meta::PairMeta,
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

    pub fn test_pair_meta_for(pair: &Pair, attribute: &str, value: &str) -> PairMeta {
        PairMeta::new(&test_keys(), pair, attribute, value)
    }

    /// returns dummy pair meta for testing
    pub fn test_pair_meta() -> PairMeta {
        test_pair_meta_for(&test_pair(), &test_attribute(), &test_value())
    }

    /// dummy pair meta, same as test_pair_meta()
    pub fn test_pair_meta_a() -> PairMeta {
        test_pair_meta()
    }

    /// returns dummy pair meta for testing against the same pair as test_pair_meta_a
    pub fn test_pair_meta_b() -> PairMeta {
        test_pair_meta_for(&test_pair(), &test_attribute_b(), &test_value_b())
    }

    #[test]
    /// smoke test PairMeta::new()
    fn new() {
        test_pair_meta();
    }

    #[test]
    /// test meta.pair()
    fn pair() {
        assert_eq!(test_pair_meta().pair_hash(), test_pair().key());
    }

    #[test]
    /// test meta.attribute()
    fn attribute() {
        assert_eq!(test_pair_meta().attribute(), test_attribute());
    }

    #[test]
    /// test meta.value()
    fn value() {
        assert_eq!(test_pair_meta().value(), test_value());
    }

    #[test]
    /// test meta.source()
    fn source() {
        assert_eq!(test_pair_meta().source(), test_keys().node_id());
    }

    #[test]
    /// test that we can sort pair metas with cmp
    fn cmp() {
        let p1 = test_pair_a();
        let p2 = test_pair_b();

        // basic ordering
        let m_1ax = PairMeta::new(&test_keys(), &p1, "a", "x");
        let m_1ay = PairMeta::new(&test_keys(), &p1, "a", "y");
        let m_1bx = PairMeta::new(&test_keys(), &p1, "b", "x");
        let m_2ax = PairMeta::new(&test_keys(), &p2, "a", "x");

        // sort by pair key
        assert_eq!(Ordering::Less, m_1ax.cmp(&m_2ax));
        assert_eq!(Ordering::Equal, m_1ax.cmp(&m_1ax));
        assert_eq!(Ordering::Greater, m_2ax.cmp(&m_1ax));
        assert_eq!(Ordering::Less, m_1ay.cmp(&m_2ax));

        // pair key with operators
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
        let pair_meta = test_pair_meta();
        let expected = "{\"pair_hash\":\"QmawqBCVVap9KdaakqEHF4JzUjjLhmR7DpM5jgJko8j1rA\",\"attribute\":\"meta-attribute\",\"value\":\"meta value\",\"source\":\"test node id\"}";

        assert_eq!(expected.to_string(), pair_meta.to_json().unwrap());
        assert_eq!(pair_meta, PairMeta::from_json(&expected).unwrap());
        assert_eq!(
            pair_meta,
            PairMeta::from_json(&pair_meta.to_json().unwrap()).unwrap()
        );
    }
}
