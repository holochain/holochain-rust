use crate::nucleus::ribosome::api::ZomeApiFunction;
use holochain_core_types::error::HolochainError;
use std::str::FromStr;
use wasmi::{
    self, Error as InterpreterError, FuncInstance, FuncRef, ImportsBuilder, ModuleImportResolver,
    ModuleInstance, ModuleRef, NopExternals, Signature, ValueType,
};

/// Creates WASMi module from gibven WASM binary
pub fn wasmi_factory(wasm: Vec<u8>) -> Result<ModuleRef, HolochainError> {
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
    ModuleInstance::new(&module, &imports)
        .expect("Failed to instantiate module")
        .run_start(&mut NopExternals)
        .map_err(|_| HolochainError::RibosomeFailed("Module failed to start".to_string()))
}
