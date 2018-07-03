// use multihash::Multihash;
use multihash::{encode, Hash};
use rust_base58::{ToBase58};

/// magic all in one fn, take a serialized something + hash type and get a hashed b58 string back
pub fn str_to_b58_hash(s: &str, hash_type: Hash) -> String {
    encode(hash_type, s.as_bytes()).unwrap().to_base58()
}
