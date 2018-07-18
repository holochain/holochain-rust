use hash_table::pair::Pair;
use agent::keys::Keys;
use multihash::Hash;
use hash::serializable_to_b58_hash;

#[derive(Serialize, Debug, Clone, PartialEq)]
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
    pair: String,
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

impl PairMeta {

    /// Builds a new PairMeta from EAV and agent keys, where E is an existing Pair
    /// @TODO need a `from()` to build a local meta from incoming network messages
    /// @see https://github.com/holochain/holochain-rust/issues/140
    pub fn new(keys: &Keys, pair: &Pair, attribute: &str, value: &str) -> PairMeta {
        PairMeta{
            pair: pair.key(),
            attribute: attribute.into(),
            value: value.into(),
            source: keys.node_id().into(),
        }
    }

    /// getter for pair clone
    pub fn pair(&self) -> String {
        self.pair.clone()
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

    /// the key for hash table lookups, e.g. table.get_meta()
    pub fn key(&self) -> String {
        serializable_to_b58_hash(&self, Hash::SHA2256)
    }

}

#[cfg(test)]
pub mod tests {

    use super::PairMeta;
    use hash_table::pair::tests::test_pair;
    use agent::keys::tests::test_keys;

    pub fn test_attribute() -> String {
        "meta-attribute".into()
    }

    pub fn test_value() -> String {
        "meta value".into()
    }

    /// returns dummy pair meta for testing
    pub fn test_pair_meta() -> PairMeta {
        PairMeta::new(&test_keys(), &test_pair(), &test_attribute(), &test_value())
    }

    #[test]
    /// smoke test PairMeta::new()
    fn new() {
        test_pair_meta();
    }

    #[test]
    /// test meta.pair()
    fn pair() {
        assert_eq!(test_pair_meta().pair(), test_pair().key());
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
}
