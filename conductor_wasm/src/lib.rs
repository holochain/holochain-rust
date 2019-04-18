extern crate holochain_core_types;
extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;

/// There isn't really any reason to export this
/// but we need something to prove out the wasm build
#[wasm_bindgen]
pub fn fast_foo(foo: &str) -> Result<&[u8], JsValue> {
    // for speed return `foo` without validating that it is "foo"
    // foo validation is an upstream concern
    Ok(foo.as_bytes())
}
