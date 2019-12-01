//! holochain_core_types::dna::wasm is a module for managing webassembly code
//!  - within the in-memory dna struct
//!  - and serialized to json
use backtrace::Backtrace;

use crate::error::HolochainError;
use base64;
use serde::{
    self,
    de::{Deserializer, SeqAccess, Visitor},
    ser::{Error, Serializer, SerializeSeq},
};
use std::{
    cmp,
    fmt,
    hash::{Hash, Hasher},
    io::{Read, BufReader},
    ops::Deref,
    sync::{Arc, RwLock},
};
use wasmi::Module;
use flate2::{Compression, bufread::{GzEncoder, GzDecoder}};

/// Wrapper around wasmi::Module since it does not implement Clone, Debug, PartialEq, Eq,
/// which are all needed to add it to the DnaWasm below, and hence to the state.
#[derive(Clone)]
pub struct ModuleArc(Arc<Module>);
impl ModuleArc {
    pub fn new(module: Module) -> Self {
        ModuleArc(Arc::new(module))
    }
}
impl PartialEq for ModuleArc {
    fn eq(&self, _other: &ModuleArc) -> bool {
        //*self == *other
        false
    }
}
impl Eq for ModuleArc {}
impl Deref for ModuleArc {
    type Target = Arc<Module>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl fmt::Debug for ModuleArc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ModuleMutex")
    }
}

/// Private helper for converting binary WebAssembly into base64 serialized string sequence.
/// Encodes to Gzip compressed, base-64 encoded String, or [String, ...]
fn _vec_u8_to_b64_str<S>(data: &Arc<Vec<u8>>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let buf_reader = BufReader::new(data.as_slice());
    let mut gz = GzEncoder::new(buf_reader, Compression::default());
    let mut buf = Vec::new();
    gz.read_to_end(&mut buf).map_err(S::Error::custom)?;
    let b64 = base64::encode(&buf);
    let cnt = ( b64.len() + 127 ) / 128;
    println!("WASM of {} bytes gzipped to {} bytes, base-64 encoded to {} bytes, {} rows",
             data.len(), buf.len(), b64.len(), &cnt);
    if cnt <= 1 {
        // For small WASMs (and backward-compatibility) emit them as a simple *un-compressed* "string"
        let b64 = base64::encode(data.as_ref());
        println!("Encoding {}-byte base-64 uncompressed WASM to String", b64.len());
        s.serialize_str(&b64)
    } else {
        // Output the base-64 encoded compressed WASM in (1024*5/4)/10 == 128-symbol chunks
        let mut seq = s.serialize_seq(Some(cnt)).map_err(S::Error::custom)?;
        println!("Encoding {}-byte base-64 gzipped WASM into {} String rows", b64.len(), &cnt);
        let mut cur: &str = b64.as_ref();
        while ! cur.is_empty() {
            let (chunk, rest) = cur.split_at(cmp::min( 128, cur.len()));
            seq.serialize_element(chunk).map_err(S::Error::custom)?;
            cur = rest;
        };
        seq.end()
    }
}

/// Private helper for converting base64 string into binary WebAssembly.  Decodes base-64 encoded
/// String (optionally Gzip-compressed for backward-compatibility), or Gzip-compressed [String, ...]
fn _b64_str_to_vec_u8<'de, D>(d: D) -> Result<Arc<Vec<u8>>, D::Error>
where
    D: Deserializer<'de>,
{
    /// visitor struct needed for serde deserialization
    struct Z;

    impl<'de> Visitor<'de> for Z {
        type Value = Vec<u8>;

        /// we only want to accept strings
        fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            formatter.write_str("base-64 encoded string, or [string, ...] ")
        }

        /// if we get a String, try to base-64 decode straight into binary WASM
        fn visit_str<E>(self, value: &str) -> Result<Vec<u8>, E>
        where
            E: serde::de::Error,
        {
            println!("Decoding {}-symbol base-64 raw WASM String", value.len());
            base64::decode(value).map_err(serde::de::Error::custom)
        }

        /// If we got a [String, ...], decode base-64 and uncompress into binary WASM
        fn visit_seq<A>(self, mut seq: A) -> Result<Vec<u8>, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut compressed: Vec<u8> = vec![];
            while let Some(elem) = seq.next_element::<&[u8]>()? {
                println!("Decoding {}-symbol base-64 raw WASM row", elem.len());
                compressed.extend_from_slice( // inefficient but WASM loading is rare
                    &base64::decode(elem)
                        .map_err(serde::de::Error::custom)?
                );
            }
            let mut gz = GzDecoder::new(compressed.as_slice());
            let mut value: Vec<u8> = vec![];
            gz.read_to_end(&mut value).map_err(serde::de::Error::custom)?;
            Ok(value)
        }
    }

    Ok(Arc::new(d.deserialize_any(Z)?))
}

/// Represents web assembly code.
#[derive(Serialize, Deserialize, Clone)]
pub struct DnaWasm {
    /// The actual binary WebAssembly bytecode goes here.
    #[serde(
        serialize_with = "_vec_u8_to_b64_str",
        deserialize_with = "_b64_str_to_vec_u8"
    )]
    pub code: Arc<Vec<u8>>,

    /// This is a transient parsed representation of the binary code.
    /// This gets only create once from the code and then cached inside this RwLock
    /// because creation of these WASMi modules from bytes is expensive.
    #[serde(skip, default = "empty_module")]
    module: Arc<RwLock<Option<ModuleArc>>>,
}

impl DnaWasm {
    /// Provide basic placeholder for wasm entries in dna structs, used for testing only.
    pub fn new_invalid() -> Self {
        debug!(
            "DnaWasm::new_invalid() called from:\n{:?}",
            Backtrace::new()
        );
        DnaWasm {
            code: Arc::new(vec![]),
            module: empty_module(),
        }
    }
}

fn empty_module() -> Arc<RwLock<Option<ModuleArc>>> {
    Arc::new(RwLock::new(None))
}

impl fmt::Debug for DnaWasm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<<<DNA WASM CODE>>>")
    }
}

impl PartialEq for DnaWasm {
    fn eq(&self, other: &DnaWasm) -> bool {
        self.code == other.code
    }
}

impl Hash for DnaWasm {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.code.hash(state);
    }
}

impl DnaWasm {
    /// Creates a new instance from given WASM binary
    pub fn from_bytes(wasm: Vec<u8>) -> Self {
        DnaWasm {
            code: Arc::new(wasm),
            module: empty_module(),
        }
    }

    /// This returns a parsed WASMi representation of the code, ready to be
    /// run in a WASMi ModuleInstance.
    /// The first call will create the module from the binary.
    pub fn get_wasm_module(&self) -> Result<ModuleArc, HolochainError> {
        if self.module.read().unwrap().is_none() {
            self.create_module()?;
        }

        Ok(self.module.read().unwrap().as_ref().unwrap().clone())
    }

    fn create_module(&self) -> Result<(), HolochainError> {
        let module = wasmi::Module::from_buffer(&*self.code).map_err(|e| {
            debug!(
                "DnaWasm could not create a wasmi::Module from code bytes! Error: {:?}",
                e
            );
            debug!("Unparsable bytes: {:?}", *self.code);
            HolochainError::ErrorGeneric(e.into())
        })?;
        let module_arc = ModuleArc::new(module);
        let mut lock = self.module.write().unwrap();
        *lock = Some(module_arc);
        Ok(())
    }
}
