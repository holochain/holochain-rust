use crate::{
    context::Context,
    NEW_RELIC_LICENSE_KEY,
};
use std::convert::TryInto;
use holochain_wasm_types::ZomeApiResult;
use holochain_core_types::error::HolochainError;
use holochain_json_api::json::JsonString;
use std::{sync::Arc};
use wasmer_runtime::{func, imports, instantiate, Array, Ctx, Instance, Module, WasmPtr};
use crate::workflows::debug::invoke_debug;
use crate::workflows::commit::invoke_commit_app_entry;
use holochain_wasm_types::WasmError;

/// Creates a WASM module, that is the executable program, from a given WASM binary byte array.
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn wasm_module_factory(wasm: Arc<Vec<u8>>) -> Result<Module, HolochainError> {
    let import_object = imports! {};
    Ok(instantiate(&wasm, &import_object)
        .map_err(|e| HolochainError::from(e.to_string()))?
        .module())
}

fn parameters_json(ctx: &Ctx, ptr: WasmPtr<u8, Array>, len: u32) -> JsonString {
    match ptr.get_utf8_string(ctx.memory(0), len) {
        Some(s) => JsonString::from_json(s),
        None => JsonString::null(),
    }
}

/// Creates a runnable WASM module instance from a module reference.
/// Adds the Holochain specific API functions as imports.
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn wasm_instance_factory(
    context: Arc<Context>,
    module: &Module,
) -> Result<Instance, HolochainError> {
    let import_object = imports! {
        "env" => {
            "hc_debug" => func!(|ctx: &mut Ctx, ptr: WasmPtr<u8, Array>, len: u32| -> ZomeApiResult {
                Ok(holochain_wasmer_host::json::to_allocation_ptr(
                    invoke_debug(context.clone(), parameters_json(ctx, ptr, len).try_into()?).map_err(|e| WasmError::Zome(e.to_string()))?.into()
                ))
            }),
            "hc_commit_entry" => func!(|ctx: &mut Ctx, ptr: WasmPtr<u8, Array>, len: u32| -> ZomeApiResult {
                    Ok(holochain_wasmer_host::json::to_allocation_ptr(
                        invoke_commit_app_entry(
                            context.clone(),
                            parameters_json(ctx, ptr, len).try_into()?
                        ).map_err(|e| WasmError::Zome(e.to_string()))?.into()
                    ))
            }),
        },
    };
    Ok(module
        .instantiate(&import_object)
        .map_err(|e| HolochainError::from(e.to_string()))?)
}
