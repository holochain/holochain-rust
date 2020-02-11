use crate::{wasm_engine::api::ZomeApiFunction, NEW_RELIC_LICENSE_KEY};
use holochain_core_types::error::HolochainError;
use holochain_json_api::json::JsonString;
use holochain_wasm_utils::memory::allocation::WasmAllocation;
use std::{convert::TryInto, sync::Arc};
use wasmer_runtime::{
    error::RuntimeError, func, imports, instantiate, Array, Ctx, Instance, Module, WasmPtr,
};

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
pub fn wasm_instance_factory(module: &Module) -> Result<ModuleRef, HolochainError> {
    // invoke_index and resolve_func work together to enable callable host functions
    // within WASM modules, which is how the core API functions
    // read about the Externals trait for more detail

    // Correlate the names of the core ZomeApiFunction's with their indexes
    // and declare its function signature (which is always the same)
    struct RuntimeModuleImportResolver;
    impl ModuleImportResolver for RuntimeModuleImportResolver {
        fn resolve_func(
            &self,
            field_name: &str,
            _signature: &Signature,
        ) -> Result<FuncRef, InterpreterError> {
            let api_fn = match ZomeApiFunction::from_str(&field_name) {
                Ok(api_fn) => api_fn,
                Err(_) => {
                    return Err(InterpreterError::Function(format!(
                        "host module doesn't export function with name {}",
                        field_name
                    )));
                }
            };

            match api_fn {
                // Abort is a way to receive useful debug info from
                // assemblyscript memory allocators, see enum definition for function signature
                ZomeApiFunction::Abort => Ok(FuncInstance::alloc_host(
                    Signature::new(
                        &[
                            ValueType::I64,
                            ValueType::I64,
                            ValueType::I64,
                            ValueType::I64,
                        ][..],
                        None,
                    ),
                    api_fn as usize,
                )),
                // All of our Zome API Functions have the same signature
                _ => Ok(FuncInstance::alloc_host(
                    Signature::new(&[ValueType::I64][..], Some(ValueType::I64)),
                    api_fn as usize,
                )),
            }
        }
    }
}

/// Creates a runnable WASM module instance from a module reference.
/// Adds the Holochain specific API functions as imports.
pub fn wasm_instance_factory(
    context: Arc<Context>,
    module: &Module,
) -> Result<Instance, HolochainError> {
    let import_object = imports! {
        "env" => {
            "hc_debug" => func!(|ctx: &mut Ctx, ptr: WasmPtr<u8, Array>, len: u32| -> ZomeApiResult {
                invoke_debug(context.clone(), parameters_json(ctx, ptr, len).try_into()?)
            }),
            "hc_commit_entry" => func!(|ctx: &mut Ctx, ptr: WasmPtr<u8, Array>, len: u32| -> ZomeApiResult {
                WasmAllocation::from(JsonString::from(
                    invoke_commit_app_entry(
                        context,
                        parameters_json(ctx, ptr, len).try_into()?
                    ).map_err(|e| RuntimeError::Trap{ msg: e.to_string().into_boxed_str() })?
                )).into()
            }),
        },
    };
    Ok(module
        .instantiate(&import_object)
        .map_err(|e| HolochainError::from(e.to_string()))?)
}
