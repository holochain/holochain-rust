use crate::{
    NEW_RELIC_LICENSE_KEY,
};
use std::convert::TryInto;
use holochain_wasm_types::ZomeApiResult;
use holochain_core_types::error::HolochainError;
use holochain_json_api::json::JsonString;
use wasmer_runtime::{func, imports, instantiate, Ctx, Instance};
// use crate::workflows::debug::debug_workflow;
// use crate::workflows::commit::invoke_commit_app_entry;
use holochain_wasm_types::WasmError;
use wasmer_runtime::ImportObject;
use crate::wasm_engine::runtime::WasmCallData;
use crate::workflows::debug::invoke_debug;
use holochain_wasmer_host::AllocationPtr;

// macro_rules! zome_api_func {
//     (
//         $context:ident, $invoke_fn:ident
//     ) => {{
//         // let closure_context = std::sync::Arc::clone(&$context);
//         // let closure_call
//         func!(|ctx: &mut Ctx, guest_allocation_ptr: $crate::holochain_wasmer_host::AllocationPtr| -> ZomeApiResult {
//             let guest_bytes = holochain_wasmer_host::guest::read_from_allocation_ptr(ctx, guest_allocation_ptr)?;
//             let guest_json = JsonString::from(guest_bytes);
//
//             Ok(holochain_wasmer_host::json::to_allocation_ptr(
//                 $invoke_fn(std::sync::Arc::clone(&$context), guest_json.try_into()?).map_err(|e| WasmError::Zome(e.to_string()))?.into()
//             ))
//         })
//     }}
// }

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn wasm_imports<'a>(
    call_data: WasmCallData,
) -> Result<ImportObject, HolochainError> {
    let context = call_data.context()?;
    let debug_context = std::sync::Arc::downgrade(&context);
    Ok(imports! {
        "env" => {
            // "hc_debug" => zome_api_func!(context, invoke_debug),
            "hc_debug" => func!(move |ctx: &mut Ctx, guest_allocation_ptr: AllocationPtr| -> ZomeApiResult {
                let guest_bytes = holochain_wasmer_host::guest::read_from_allocation_ptr(ctx, guest_allocation_ptr)?;
                let guest_json = JsonString::from(guest_bytes);

                Ok(holochain_wasmer_host::json::to_allocation_ptr(
                    invoke_debug(debug_context, guest_json.try_into()?).map_err(|e| WasmError::Zome(e.to_string()))?.into()
                ))
            }),
            // "hc_commit_entry" => zome_api_func!(context, invoke_commit_app_entry),
        },
    })
}

/// Returns the WASM module, i.e. the WASM binary program code to run
/// for the given WasmCallData.
///
/// In case of a direct call, the module gets created from the WASM binary
/// inside the DirectCall specialisation for WasmCallData.
///
/// For ZomeCalls and CallbackCalls it gets the according module from the DNA.
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn instance_for_call_data(data: &WasmCallData) -> Result<Instance, HolochainError> {
    fn instance(wasm: &Vec<u8>, call_data: WasmCallData) -> Result<Instance, HolochainError> {
        Ok(instantiate(wasm, &wasm_imports(call_data)?).map_err(|e| HolochainError::from(e.to_string()))?)
    }

    let (context, zome_name) = if let WasmCallData::DirectCall(_, wasm) = data {
        return instance(&wasm, data.clone());
    } else {
        match data {
            WasmCallData::ZomeCall(d) => (d.context.clone(), d.call.zome_name.clone()),
            WasmCallData::CallbackCall(d) => (d.context.clone(), d.call.zome_name.clone()),
            WasmCallData::DirectCall(_, _) => unreachable!(),
        }
    };

    let state_lock = context.state()?;
    // @TODO caching for wasm and/or modules, just reinstance them
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

    instance(&wasm, data.clone())
}
