#[derive(Clone, Debug)]
pub struct Code {
    format: String,
    language: String,
    code: String,
}

#[derive(Clone, Debug)]
pub struct Entry {}

#[derive(Clone, Debug)]
pub struct Zome {
    entry_definitions: Vec<Entry>,
}

use std::fs::File;
#[derive(Clone, Debug, PartialEq)]
pub struct DNA {}

impl DNA {
    pub fn wasm_for_zome_function(&self, capability: &str, function_name: &str) -> Vec<u8> {
        use std::io::prelude::*;
        let mut file = File::open("src/nucleus/wasm-test/target/wasm32-unknown-unknown/release/wasm_ribosome_test.wasm").unwrap();
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).unwrap();
        return buf;
    }
}