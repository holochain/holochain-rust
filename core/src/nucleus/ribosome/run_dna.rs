use crate::{
    context::Context,
    nucleus::{
        ribosome::{api::ZomeApiFunction, memory::SinglePageManager, Runtime},
        ZomeFnCall, ZomeFnResult,
    },
};
use holochain_core_types::{
    error::{HcResult, HolochainError, RibosomeErrorCode, RibosomeReturnCode},
    json::JsonString,
};
use holochain_wasm_utils::memory_allocation::decode_encoded_allocation;
use std::{str::FromStr, sync::Arc};
use wasmi::{
    self, Error as InterpreterError, FuncInstance, FuncRef, ImportsBuilder, ModuleImportResolver,
    ModuleInstance, NopExternals, RuntimeValue, Signature, ValueType,
};

/// Executes an exposed zome function in a wasm binary.
/// Multithreaded function
/// panics if wasm binary isn't valid.
pub fn run_dna(
    dna_name: &str,
    context: Arc<Context>,
    wasm: Vec<u8>,
    zome_call: &ZomeFnCall,
    parameters: Option<Vec<u8>>,
) -> ZomeFnResult {
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
                            ValueType::I32,
                            ValueType::I32,
                            ValueType::I32,
                            ValueType::I32,
                        ][..],
                        None,
                    ),
                    api_fn as usize,
                )),
                // All of our Zome API Functions have the same signature
                _ => Ok(FuncInstance::alloc_host(
                    Signature::new(&[ValueType::I32][..], Some(ValueType::I32)),
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

    // instantiate runtime struct for passing external state data over wasm but not to wasm
    let mut runtime = Runtime {
        memory_manager: SinglePageManager::new(&wasm_instance),
        context,
        zome_call: zome_call.clone(),
        dna_name: dna_name.to_string(),
    };

    // Write input arguments in wasm memory
    // scope for mutable borrow of runtime
    let encoded_allocation_of_input: u32;
    {
        let mut_runtime = &mut runtime;
        let maybe_allocation_of_input = mut_runtime.memory_manager.write(&input_parameters);
        encoded_allocation_of_input = match maybe_allocation_of_input {
            // No allocation to write is ok
            Err(RibosomeErrorCode::ZeroSizedAllocation) => 0,
            // Any other error is memory related
            Err(err) => {
                return Err(HolochainError::RibosomeFailed(err.to_string()));
            }
            // Write successful, encode allocation
            Ok(allocation_of_input) => allocation_of_input.encode(),
        }
    }

    // scope for mutable borrow of runtime
    let returned_encoded_allocation: u32;
    {
        let mut_runtime = &mut runtime;

        // invoke function in wasm instance
        // arguments are info for wasm on how to retrieve complex input arguments
        // which have been set in memory module
        returned_encoded_allocation = wasm_instance
            .invoke_export(
                zome_call.fn_name.clone().as_str(),
                &[RuntimeValue::I32(encoded_allocation_of_input as i32)],
                mut_runtime,
            )
            .map_err(|err| HolochainError::RibosomeFailed(err.to_string()))?
            .unwrap()
            .try_into()
            .unwrap();
    }

    // Handle result returned by called zome function
    let maybe_allocation = decode_encoded_allocation(returned_encoded_allocation);
    let return_log_msg: String;
    let return_result: HcResult<JsonString>;
    match maybe_allocation {
        // Nothing in memory, return result depending on return_code received.
        Err(return_code) => {
            return_log_msg = return_code.to_string();
            return_result = match return_code {
                RibosomeReturnCode::Success => Ok(JsonString::null()),
                RibosomeReturnCode::Failure(err_code) => {
                    Err(HolochainError::RibosomeFailed(err_code.to_string()))
                }
            };
        }
        // Something in memory, try to read and return it
        Ok(valid_allocation) => {
            let result = runtime.memory_manager.read(valid_allocation);
            let maybe_zome_result = String::from_utf8(result);
            match maybe_zome_result {
                Err(err) => {
                    return_log_msg = err.to_string();
                    return_result = Err(HolochainError::RibosomeFailed(err.to_string()));
                }
                Ok(json_str) => {
                    return_log_msg = json_str.clone();
                    return_result = Ok(JsonString::from(json_str));
                }
            }
        }
    };
    // Log & done
    runtime.context.log(format!(
        "debug/zome: Zome Function '{}' returned: {}",
        zome_call.fn_name, return_log_msg,
    ));
    return return_result;
}
