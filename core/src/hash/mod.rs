// use multihash::Multihash;
use multihash::{encode, Hash};
use rust_base58::ToBase58;

/// convert bytes to a b58 hashed string
pub fn bytes_to_b58_hash(bytes: &[u8], hash_type: Hash) -> String {
    encode(hash_type, bytes).unwrap().to_base58()
}

/// magic all in one fn, take a serialized something + hash type and get a hashed b58 string back
pub fn str_to_b58_hash(s: &str, hash_type: Hash) -> String {
    bytes_to_b58_hash(s.as_bytes(), hash_type)
}

#[cfg(test)]
mod tests {
    use multihash::Hash;

    #[test]
    /// mimics tests from legacy golang holochain core hashing bytes
    fn bytes_to_b58_known_golang() {
        assert_eq!(
            super::bytes_to_b58_hash(b"test data", Hash::SHA2256),
            "QmY8Mzg9F69e5P9AoQPYat655HEhc1TVGs11tmfNSzkqh2"
        )
    }

    #[test]
    /// mimics tests from legacy golang holochain core hashing strings
    fn str_to_b58_hash_known_golang() {
        assert_eq!(
            super::str_to_b58_hash("test data", Hash::SHA2256),
            "QmY8Mzg9F69e5P9AoQPYat655HEhc1TVGs11tmfNSzkqh2"
        );
    }
}
