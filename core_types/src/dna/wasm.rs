//! holochain_core_types::dna::wasm is a module for managing webassembly code
//!  - within the in-memory dna struct
//!  - and serialized to json

use base64;
use serde::{
    self,
    de::{Deserializer, Visitor},
    ser::Serializer,
};
use std::fmt;

/// Private helper for converting binary WebAssembly into base64 serialized string.
fn _vec_u8_to_b64_str<S>(data: &[u8], s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let b64 = base64::encode(data);
    s.serialize_str(&b64)
}

/// Private helper for converting base64 string into binary WebAssembly.
fn _b64_str_to_vec_u8<'de, D>(d: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    /// visitor struct needed for serde deserialization
    struct Z;

    impl<'de> Visitor<'de> for Z {
        type Value = Vec<u8>;

        /// we only want to accept strings
        fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            formatter.write_str("string")
        }

        /// if we get a string, try to base64 decode into binary
        fn visit_str<E>(self, value: &str) -> Result<Vec<u8>, E>
        where
            E: serde::de::Error,
        {
            match base64::decode(value) {
                Ok(v) => Ok(v),
                Err(e) => Err(serde::de::Error::custom(e)),
            }
        }
    }

    d.deserialize_any(Z)
}

/// Represents web assembly code.
#[derive(Serialize, Deserialize, Clone, PartialEq, Hash)]
pub struct DnaWasm {
    /// The actual binary WebAssembly bytecode goes here.
    #[serde(
        serialize_with = "_vec_u8_to_b64_str",
        deserialize_with = "_b64_str_to_vec_u8"
    )]
    pub code: Vec<u8>,
    // using a struct gives us the flexibility to extend it later
    // should we need additional properties, like:
    //pub filename: String,
}

impl Default for DnaWasm {
    /// Provide defaults for wasm entries in dna structs.
    fn default() -> Self {
        DnaWasm { code: vec![] }
    }
}

impl fmt::Debug for DnaWasm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<<<DNA WASM CODE>>>")
    }
}

impl DnaWasm {
    /// Allow sane defaults for `DnaWasm::new()`.
    pub fn new() -> Self {
        Default::default()
    }
}
