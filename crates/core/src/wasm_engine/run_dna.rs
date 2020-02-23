use crate::{
    // nucleus::ZomeFnResult,
    {
        factories::{wasm_instance_factory, wasm_module_factory},
        runtime::WasmCallData,
    },
    // NEW_RELIC_LICENSE_KEY,
};
use holochain_core_types::error::HolochainError;
use holochain_json_api::json::JsonString;

use wasmer_runtime::Module;

/// Returns the WASM module, i.e. the WASM binary program code to run
/// for the given WasmCallData.
///
/// In case of a direct call, the module gets created from the WASM binary
/// inside the DirectCall specialisation for WasmCallData.
///
/// For ZomeCalls and CallbackCalls it gets the according module from the DNA.
// #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
fn get_module(data: WasmCallData) -> Result<Module, HolochainError> {
    let (context, zome_name) = if let WasmCallData::DirectCall(_, wasm) = data {
        return Ok(wasm_module_factory(wasm)?);
    } else {
        match data {
            WasmCallData::ZomeCall(d) => (d.context.clone(), d.call.zome_name),
            WasmCallData::CallbackCall(d) => (d.context.clone(), d.call.zome_name),
            WasmCallData::DirectCall(_, _) => unreachable!(),
        }
    };

    let state_lock = context.state()?;
    let wasm = state_lock
        .nucleus()
        .dna
        .as_ref()
        .unwrap()
        .zomes
        .get(&zome_name)
        .ok_or_else(|| HolochainError::new(&format!("No Ribosome found for Zome '{}'", zome_name)))?
        .code
        .code
        .clone();

    Ok(wasm_module_factory(wasm)?)
}

/// Executes an exposed zome function in a wasm binary.
/// Multithreaded function
/// panics if wasm binary isn't valid.
#[autotrace]
// #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn run_dna<I: Into<JsonString>>(data: WasmCallData, input: I) -> ZomeFnResult {
    let wasm_module = get_module(data.clone())?;

    // instantiate runtime struct for passing external state data over wasm but not to wasm
    let mut wasm_instance = wasm_instance_factory(
        data.context()
            .map_err(|e| HolochainError::from(e.to_string()))?,
        &wasm_module,
    )?;

    let fn_name = data.fn_name();

    holochain_wasmer_host::guest::call(&mut wasm_instance, &fn_name, input);
}
