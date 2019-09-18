#![warn(unused_extern_crates)]
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_json;

pub mod mock_signing;

use crossbeam_channel::Receiver;
use holochain_conductor_api::{context_builder::ContextBuilder, error::HolochainResult, Holochain};
use holochain_core::{
    action::Action,
    context::Context,
    logger::{test_logger, TestLogger},
    nucleus::actions::call_zome_function::make_cap_request_for_call,
    signal::{Signal,signal_channel,SignalReceiver}
};
use holochain_core_types::{
   dna::{
        entry_types::{EntryTypeDef, LinkedFrom, LinksTo, Sharing},
        fn_declarations::{FnDeclaration, TraitFns},
        traits::ReservedTraitNames,
        wasm::DnaWasm,
        zome::{Config, Zome, ZomeFnDeclarations, ZomeTraits},
        Dna,
    },
    entry::entry_type::{AppEntryType, EntryType},
};
use holochain_persistence_api::{
    cas::content::AddressableContent,
};
use holochain_json_api::json::JsonString;

use holochain_net::p2p_config::P2pConfig;

use std::{
    collections::{hash_map::DefaultHasher, BTreeMap},
    fs::File,
    hash::{Hash, Hasher},
    io::prelude::*,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};
use tempfile::tempdir;
use wabt::Wat2Wasm;

/// Load WASM from filesystem
pub fn create_wasm_from_file(path: &PathBuf) -> Vec<u8> {
    let mut file = File::open(path)
        .unwrap_or_else(|err| panic!("Couldn't create WASM from file: {:?}; {}", path, err));
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();
    buf
}

/// Create DNA from WAT
pub fn create_test_dna_with_wat(zome_name: &str, wat: Option<&str>) -> Dna {
    // Default WASM code returns 1337 as integer
    let default_wat = r#"
            (module
                (memory (;0;) 1)
                (func (export "public_test_fn") (param $p0 i64) (result i64)
                    i64.const 6
                )
                (data (i32.const 0)
                    "1337.0"
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

    create_test_dna_with_wasm(zome_name, wasm_binary.as_ref().to_vec())
}

/// Prepare valid DNA struct with that WASM in a zome's capability
pub fn create_test_dna_with_wasm(zome_name: &str, wasm: Vec<u8>) -> Dna {
    let mut dna = Dna::new();
    let defs = create_test_defs_with_fn_name("public_test_fn");

    //    let mut capabilities = BTreeMap::new();
    //    capabilities.insert(cap_name.to_string(), capability);

    let mut test_entry_def = EntryTypeDef::new();
    test_entry_def.links_to.push(LinksTo {
        target_type: String::from("testEntryType"),
        link_type: String::from("test-link"),
    });

    let mut test_entry_b_def = EntryTypeDef::new();
    test_entry_b_def.linked_from.push(LinkedFrom {
        base_type: String::from("testEntryType"),
        link_type: String::from("test-link"),
    });

    let mut test_entry_c_def = EntryTypeDef::new();
    test_entry_c_def.sharing = Sharing::Private;

    let mut entry_types = BTreeMap::new();

    entry_types.insert(
        EntryType::App(AppEntryType::from("testEntryType")),
        test_entry_def,
    );
    entry_types.insert(
        EntryType::App(AppEntryType::from("testEntryTypeB")),
        test_entry_b_def,
    );
    entry_types.insert(
        EntryType::App(AppEntryType::from("testEntryTypeC")),
        test_entry_c_def,
    );

    let mut zome = Zome::new(
        "some zome description",
        &Config::new(),
        &entry_types,
        &defs.0,
        &defs.1,
        &DnaWasm::from_bytes(wasm),
    );

    let mut trait_fns = TraitFns::new();
    trait_fns.functions.push("public_test_fn".to_string());
    zome.traits
        .insert(ReservedTraitNames::Public.as_str().to_string(), trait_fns);
    dna.zomes.insert(zome_name.to_string(), zome);
    dna.name = "TestApp".into();
    dna.uuid = "8ed84a02-a0e6-4c8c-a752-34828e302986".into();
    dna
}

pub fn create_test_defs_with_fn_name(fn_name: &str) -> (ZomeFnDeclarations, ZomeTraits) {
    let mut trait_fns = TraitFns::new();
    let mut fn_decl = FnDeclaration::new();
    fn_decl.name = String::from(fn_name);
    trait_fns.functions.push(String::from(fn_name));
    let mut traits = BTreeMap::new();
    traits.insert(ReservedTraitNames::Public.as_str().to_string(), trait_fns);

    let mut functions = Vec::new();
    functions.push(fn_decl);
    (functions, traits)
}

pub fn create_test_defs_with_fn_names(fn_names: Vec<String>) -> (ZomeFnDeclarations, ZomeTraits) {
    let mut trait_fns = TraitFns::new();
    let mut functions = Vec::new();
    for fn_name in fn_names {
        let mut fn_decl = FnDeclaration::new();
        fn_decl.name = fn_name.clone();
        functions.push(fn_decl);
        trait_fns.functions.push(fn_name.clone());
    }

    let mut traits = BTreeMap::new();
    traits.insert(ReservedTraitNames::Public.as_str().to_string(), trait_fns);

    (functions, traits)
}

/// Prepare valid DNA struct with that WASM in a zome's capability
pub fn create_test_dna_with_defs(
    zome_name: &str,
    defs: (ZomeFnDeclarations, ZomeTraits),
    wasm: &[u8],
) -> Dna {
    let mut dna = Dna::new();
    let etypedef = EntryTypeDef::new();
    let mut entry_types = BTreeMap::new();
    entry_types.insert("testEntryType".into(), etypedef);
    let zome = Zome::new(
        "some zome description",
        &Config::new(),
        &entry_types,
        &defs.0,
        &defs.1,
        &DnaWasm::from_bytes(wasm.to_owned()),
    );

    dna.zomes.insert(zome_name.to_string(), zome);
    dna.name = "TestApp".into();
    dna.uuid = "8ed84a02-a0e6-4c8c-a752-34828e302986".into();
    dna
}

pub fn create_arbitrary_test_dna() -> Dna {
    let wat = r#"
    (module
     (memory 1)
     (export "memory" (memory 0))
     (export "public_test_fn" (func $func0))
     (func $func0 (param $p0 i64) (result i64)
           i64.const 16
           )
     (data (i32.const 0)
           "{\"holo\":\"world\"}"
           )
     )
    "#;
    create_test_dna_with_wat("test_zome", Some(wat))
}

#[cfg_attr(tarpaulin, skip)]
pub fn test_context_and_logger_with_network_name(
    agent_name: &str,
    network_name: Option<&str>,
) -> (Arc<Context>, Arc<Mutex<TestLogger>>) {
    let (signal,_) = signal_channel();
    let agent = mock_signing::registered_test_agent(agent_name);
    let logger = test_logger();
    (
        Arc::new({
            let mut builder = ContextBuilder::new()
                .with_agent(agent.clone())
                .with_file_storage(tempdir().unwrap().path().to_str().unwrap())
                .expect("Tempdir must be accessible")
                .with_conductor_api(mock_signing::mock_conductor_api(agent))
                .with_signals(signal);
            if let Some(network_name) = network_name {
                let config = P2pConfig::new_with_memory_backend(network_name);
                builder = builder.with_p2p_config(config);
            }
            builder
                .with_instance_name("test_context_instance")
                .spawn()
        }),
        logger,
    )
}

pub fn test_context_and_logger_with_network_name_and_signal(
    agent_name: &str,
    network_name: Option<&str>,
) -> (Arc<Context>, Arc<Mutex<TestLogger>>,SignalReceiver) {
    let (signal,reciever) = signal_channel();
    let agent = mock_signing::registered_test_agent(agent_name);
    let logger = test_logger();
    (
        Arc::new({
            let mut builder = ContextBuilder::new()
                .with_agent(agent.clone())
                .with_file_storage(tempdir().unwrap().path().to_str().unwrap())
                .expect("Tempdir must be accessible")
                .with_conductor_api(mock_signing::mock_conductor_api(agent))
                .with_signals(signal);
            if let Some(network_name) = network_name {
                let config = P2pConfig::new_with_memory_backend(network_name);
                builder = builder.with_p2p_config(config);
            }
            builder
                .with_instance_name("test_context_instance")
                .spawn()
        }),
        logger,
        reciever
    )
}

#[cfg_attr(tarpaulin, skip)]
pub fn test_context_and_logger(agent_name: &str) -> (Arc<Context>, Arc<Mutex<TestLogger>>) {
    test_context_and_logger_with_network_name(agent_name, None)
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

// Function called at start of all unit tests:
//   Startup holochain and do a call on the specified wasm function.
pub fn hc_setup_and_call_zome_fn<J: Into<JsonString>>(
    wasm_path: &PathBuf,
    fn_name: &str,
    params: J,
) -> HolochainResult<JsonString> {
    // Setup the holochain instance
    let wasm = create_wasm_from_file(wasm_path);
    let defs = create_test_defs_with_fn_name(fn_name);
    let dna = create_test_dna_with_defs("test_zome", defs, &wasm);

    let context = create_test_context("alex");
    let mut hc = Holochain::new(dna.clone(), context.clone()).unwrap();

    let params_string = String::from(params.into());
    let cap_request = make_cap_request_for_call(
        context.clone(),
        context.clone().agent_id.address(),
        fn_name,
        JsonString::from_json(&params_string.clone()),
    );

    // Run the holochain instance
    hc.start().expect("couldn't start");
    // Call the exposed wasm function
    return hc.call("test_zome", cap_request, fn_name, &params_string);
}

/// create a test context and TestLogger pair so we can use the logger in assertions
pub fn create_test_context(agent_name: &str) -> Arc<Context> {
    let agent = mock_signing::registered_test_agent(agent_name);
    Arc::new(
        ContextBuilder::new()
            .with_agent(agent.clone())
            .with_file_storage(tempdir().unwrap().path().to_str().unwrap())
            .expect("Tempdir must be accessible")
            .with_conductor_api(mock_signing::mock_conductor_api(agent))
            .with_instance_name("fake_instance_name")
            .spawn(),
    )
}

// @TODO this is a first attempt at replacing history.len() tests
// @see https://github.com/holochain/holochain-rust/issues/195
pub fn expect_action<F>(rx: &Receiver<Signal>, f: F) -> Result<Action, String>
where
    F: Fn(&Action) -> bool,
{
    let timeout = 1000;
    loop {
        match rx
            .recv_timeout(Duration::from_millis(timeout))
            .map_err(|e| e.to_string())?
        {
            Signal::Trace(aw) => {
                let action = aw.action().clone();
                if f(&action) {
                    return Ok(action);
                }
            }
            _ => continue,
        }
    }
}
