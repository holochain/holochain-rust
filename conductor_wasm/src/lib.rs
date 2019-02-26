extern crate holochain_core_types;
extern crate wasm_bindgen;

use holochain_core_types::agent::KeyBuffer;

use wasm_bindgen::prelude::*;

/// There isn't really any reason to export this
/// but we need something to prove out the wasm build
#[wasm_bindgen]
pub fn parse_agent_id(agent_id: &str) -> Result<Vec<u8>, JsValue> {
    let kb =
        KeyBuffer::with_corrected(agent_id).map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
    let mut out = kb.get_sig().to_vec();
    out.extend_from_slice(kb.get_enc());
    Ok(out)
}
