extern crate holochain_core;
extern crate holochain_dna;
extern crate wabt;

use holochain_core::*;
use holochain_dna::wasm::DnaWasm;
use holochain_dna::zome::capabilities::Capability;
use holochain_dna::zome::Zome;
use holochain_dna::Dna;
use wabt::Wat2Wasm;

use std::fs::File;

pub fn test_wasm_from_file(fname: &str) -> Vec<u8> {
    use std::io::prelude::*;
    let mut file = File::open(fname).unwrap();
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();
    buf
}

pub fn create_test_dna_with_wat(wat: Option<&str>) -> Dna {
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
    let wat_str = match wat {
        None => default_wat.as_str(),
        Some(w) => w,
    };
    // Test WASM code that returns 1337 as integer

    let wasm_binary = Wat2Wasm::new()
        .canonicalize_lebs(false)
        .write_debug_names(true)
        .convert(wat_str)
        .unwrap();

    create_test_dna_with_wasm(wasm_binary.as_ref().to_vec())
}

pub fn create_test_dna_with_wasm(wasm: Vec<u8>) -> Dna {
    // Prepare valid DNA struct with that WASM in a zome's capability:
    let mut dna = Dna::new();
    let mut zome = Zome::new();
    let mut capability = Capability::new();
    capability.name = "test_cap".to_string();
    capability.code = DnaWasm { code: wasm };
    zome.name = "test_zome".to_string();
    zome.capabilities.push(capability);
    dna.zomes.push(zome);
    dna
}
