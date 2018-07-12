use hash_table::pair::Pair;
use agent::keys::Keys;
use multihash::Hash;
use hash::serializable_to_b58_hash;

#[derive(Serialize, Debug, Clone)]
pub struct PairMeta {
    pair: String,
    attribute: String,
    value: String,
    // txn: String,
    source: String,
    // signature: String,
}

impl PairMeta {

    pub fn new(keys: &Keys, pair: &Pair, attribute: &str, value: &str) -> PairMeta {
        PairMeta{
            pair: pair.key(),
            attribute: attribute.into(),
            value: value.into(),
            source: keys.node_id().into(),
        }
    }

    pub fn key(&self) -> String {
        serializable_to_b58_hash(&self, Hash::SHA2256)
    }

}
