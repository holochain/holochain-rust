extern crate holochain_agent;
extern crate holochain_core;
extern crate holochain_dna;
extern crate wabt;

use holochain_agent::Agent;
use holochain_core::{context::Context, logger::Logger, persister::SimplePersister};
use holochain_dna::{
    wasm::DnaWasm,
    zome::{capabilities::Capability, Config, Zome},
    Dna,
};
use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    fmt,
    fs::File,
    hash::{Hash, Hasher},
    io::prelude::*,
    sync::{Arc, Mutex},
};
use wabt::Wat2Wasm;

/// Load WASM from filesystem
pub fn create_wasm_from_file(fname: &str) -> Vec<u8> {
    let mut file = File::open(fname).unwrap();
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();
    buf
}

/// Create DNA from WAT
pub fn create_test_dna_with_wat(zome_name: &str, cap_name: &str, wat: Option<&str>) -> Dna {
    // Default WASM code returns 1337 as integer
    let default_wat = r#"
            (module
                (memory (;0;) 17)
                (func (export "main") (param $p0 i32) (result i32)
                    i32.const 4
                )
                (data (i32.const 0)
                    "1337"
                )
                (export "memory" (memory 0))
            )
        "#;
    let wat_str = wat.unwrap_or_else(|| &default_wat);

    // Test WASM code that returns 1337 as integer
    let wasm_binary = Wat2Wasm::new()
        .canonicalize_lebs(false)
        .write_debug_names(true)
        .convert(wat_str)
        .unwrap();

    create_test_dna_with_wasm(zome_name, cap_name, wasm_binary.as_ref().to_vec())
}

/// Prepare valid DNA struct with that WASM in a zome's capability
pub fn create_test_dna_with_wasm(zome_name: &str, cap_name: &str, wasm: Vec<u8>) -> Dna {
    let mut dna = Dna::new();
    let mut capability = Capability::new();
    capability.code = DnaWasm { code: wasm };

    let mut capabilities = HashMap::new();
    capabilities.insert(cap_name.to_string(), capability);

    let zome = Zome::new(
        "some zome description",
        &Config::new(),
        &HashMap::new(),
        &capabilities,
    );

    // zome.capabilities.push(capability);
    dna.zomes.insert(zome_name.to_string(), zome);
    dna.name = "TestApp".into();
    dna.uuid = "8ed84a02-a0e6-4c8c-a752-34828e302986".into();
    dna
}

#[derive(Clone)]
pub struct TestLogger {
    pub log: Vec<String>,
}

impl Logger for TestLogger {
    fn log(&mut self, msg: String) {
        self.log.push(msg);
    }
}

// trying to get a way to print out what has been logged for tests without a read function.
// this currently fails
impl fmt::Debug for TestLogger {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.log)
    }
}

pub fn test_logger() -> Arc<Mutex<TestLogger>> {
    Arc::new(Mutex::new(TestLogger { log: Vec::new() }))
}

pub fn test_context_and_logger(agent_name: &str) -> (Arc<Context>, Arc<Mutex<TestLogger>>) {
    let agent = Agent::from_string(agent_name.to_string());
    let logger = test_logger();
    (
        Arc::new(Context {
            agent,
            logger: logger.clone(),
            persister: Arc::new(Mutex::new(SimplePersister::new())),
        }),
        logger,
    )
}

pub fn test_context(agent_name: &str) -> Arc<Context> {
    let (context, _) = test_context_and_logger(agent_name);
    context
}

/// calculates the native Rust hash
/// has nothing to do with our hashing e.g. multihash
/// @see https://doc.rust-lang.org/std/hash/index.html
pub fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}
