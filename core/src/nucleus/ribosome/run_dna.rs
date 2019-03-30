use crate::nucleus::{
    ribosome::{
        api::ZomeApiFunction,
        memory::WasmPageManager,
        runtime::{Runtime, WasmCallData},
    },
    ZomeFnResult,
};
use holochain_core_types::{
    error::{
        HcResult, HolochainError, RibosomeEncodedValue, RibosomeEncodingBits, RibosomeRuntimeBits,
    },
    json::JsonString,
};
use holochain_wasm_utils::memory::allocation::{AllocationError, WasmAllocation};
use std::{convert::TryFrom, str::FromStr};
use wasmi::{
    self, Error as InterpreterError, FuncInstance, FuncRef, ImportsBuilder, ModuleImportResolver,
    ModuleInstance, NopExternals, RuntimeValue, Signature, ValueType,
};

/// Executes an exposed zome function in a wasm binary.
/// Multithreaded function
/// panics if wasm binary isn't valid.
pub fn run_dna(wasm: Vec<u8>, parameters: Option<Vec<u8>>, data: WasmCallData) -> ZomeFnResult {
    // Create wasm module from wasm binary
    let module =
        wasmi::Module::from_buffer(wasm).map_err(|e| HolochainError::ErrorGeneric(e.into()))?;

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

    // Create Imports with previously described Resolver
    let mut imports = ImportsBuilder::new();
    imports.push_resolver("env", &RuntimeModuleImportResolver);

    // Create module instance from wasm module, and start it if start is defined
    let wasm_instance = ModuleInstance::new(&module, &imports)
        .expect("Failed to instantiate module")
        .run_start(&mut NopExternals)
        .map_err(|_| HolochainError::RibosomeFailed("Module failed to start".to_string()))?;

    // write input arguments for module call in memory Buffer
    let input_parameters: Vec<_> = parameters.unwrap_or_default();

    let fn_name = data.fn_name();
    // instantiate runtime struct for passing external state data over wasm but not to wasm
    let mut runtime = Runtime {
        memory_manager: WasmPageManager::new(&wasm_instance),
        data,
    };

    // Write input arguments in wasm memory
    // scope for mutable borrow of runtime
    let encoded_allocation_of_input: RibosomeEncodingBits = {
        let mut_runtime = &mut runtime;
        let maybe_allocation = mut_runtime.memory_manager.write(&input_parameters);

        match maybe_allocation {
            // No allocation to write is ok
            Err(AllocationError::ZeroLength) => RibosomeEncodedValue::Success.into(),
            // Any other error is memory related
            Err(err) => {
                return Err(HolochainError::RibosomeFailed(format!(
                    "WASM Memory issue: {:?}",
                    err
                )));
            }
            // Write successful, encode allocation
            Ok(allocation) => RibosomeEncodedValue::from(allocation).into(),
        }
    };

    // scope for mutable borrow of runtime
    let returned_encoding: RibosomeEncodingBits = {
        let mut_runtime = &mut runtime;

        // Try installing a custom panic handler.
        // HDK-rust implements a function __install_panic_handler that reroutes output of
        // PanicInfo to hdk::debug.
        // Try calling it but fail silently if this function is not there.
        let _ = wasm_instance.invoke_export("__install_panic_handler", &[], mut_runtime);
        // invoke function in wasm instance
        // arguments are info for wasm on how to retrieve complex input arguments
        // which have been set in memory module
        wasm_instance
            .invoke_export(
                &fn_name,
                &[RuntimeValue::I64(
                    RibosomeEncodingBits::from(encoded_allocation_of_input) as RibosomeRuntimeBits,
                )],
                mut_runtime,
            )
            .map_err(|err| {
                HolochainError::RibosomeFailed(format!("WASM invocation failed: {}", err))
            })?
            .unwrap()
            .try_into() // Option<_>
            .ok_or_else(|| HolochainError::RibosomeFailed("WASM return value missing".to_owned()))?
    };

    // Handle result returned by called zome function
    let return_code = RibosomeEncodedValue::from(returned_encoding);

    let return_log_msg: String;
    let return_result: HcResult<JsonString>;

    match return_code.clone() {
        RibosomeEncodedValue::Success => {
            return_log_msg = return_code.to_string();
            return_result = Ok(JsonString::null());
        }

        RibosomeEncodedValue::Failure(err_code) => {
            return_log_msg = return_code.to_string();
            return_result = Err(HolochainError::RibosomeFailed(format!(
                "Zome function failure: {}",
                err_code.as_str()
            )));
        }

        RibosomeEncodedValue::Allocation(ribosome_allocation) => {
            match WasmAllocation::try_from(ribosome_allocation) {
                Ok(allocation) => {
                    let result = runtime.memory_manager.read(allocation);
                    match String::from_utf8(result) {
                        Ok(json_string) => {
                            return_log_msg = json_string.clone();
                            return_result = Ok(JsonString::from_json(&json_string));
                        }
                        Err(err) => {
                            return_log_msg = err.to_string();
                            return_result = Err(HolochainError::RibosomeFailed(format!(
                                "WASM failed to return value: {}",
                                err
                            )));
                        }
                    }
                }
                Err(allocation_error) => {
                    return_log_msg = String::from(allocation_error.clone());
                    return_result = Err(HolochainError::RibosomeFailed(format!(
                        "WASM return value allocation failed: {:?}",
                        allocation_error,
                    )));
                }
            }
        }
    };

    // Log & done
    // @TODO make this more sophisticated (truncation or something)
    // right now we have tests that return multiple wasm pages (64k+ bytes) so this is very spammy
    // runtime.context.log(format!(
    //     "debug/zome: Zome Function '{}' returned: {}",
    //     zome_call.fn_name, return_log_msg,
    // ));
    let _ = return_log_msg;
    return return_result;
}
