extern crate holochain_core;
extern crate holochain_dna;
extern crate wabt;

use holochain_core::*;
use holochain_dna::{
    wasm::DnaWasm, zome::{capabilities::Capability, Zome}, Dna,
};
use std::{fs::File, io::prelude::*};
use wabt::Wat2Wasm;

/// Load WASM from filesystem
pub fn create_wasm_from_file(fname: &str) -> Vec<u8> {
    let mut file = File::open(fname).unwrap();
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();
    buf
}

/// Create DNA from WAT
pub fn create_test_dna_with_wat(zome_name: String, cap_name: String, wat: Option<&str>) -> Dna {
    // Default WASM code returns 1337 as integer
    let default_wat = format!(
        r#"
                (module
                    (memory (;0;) 17)
                    (func (export "main_dispatch") (param $p0 i32) (param $p1 i32) (result i32)
                        i32.const 4
                    )
                    (data (i32.const {})
                        "1337"
                    )
                    (export "memory" (memory 0))
                )
            "#,
        nucleus::ribosome::RESULT_OFFSET
    );
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
pub fn create_test_dna_with_wasm(zome_name: String, cap_name: String, wasm: Vec<u8>) -> Dna {
    let mut dna = Dna::new();
    let mut zome = Zome::new();
    let mut capability = Capability::new();
    capability.name = cap_name;
    capability.code = DnaWasm { code: wasm };
    zome.name = zome_name;
    zome.capabilities.push(capability);
    dna.zomes.push(zome);
    dna
}
