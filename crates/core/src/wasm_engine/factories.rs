use crate::{wasm_engine::api::ZomeApiFunction, NEW_RELIC_LICENSE_KEY};
use holochain_core_types::error::HolochainError;
use std::{str::FromStr, sync::Arc};
use wasmi::{
    self, Error as InterpreterError, FuncInstance, FuncRef, ImportsBuilder, Module,
    ModuleImportResolver, ModuleInstance, ModuleRef, NopExternals, Signature, ValueType,
};

/// Creates a WASM module, that is the executable program, from a given WASM binary byte array.
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn wasm_module_factory(wasm: Arc<Vec<u8>>) -> Result<Module, HolochainError> {
    wasmi::Module::from_buffer(&*wasm).map_err(|e| HolochainError::ErrorGeneric(e.into()))
}

/// Creates a runnable WASM module instance from a module reference.
/// Adds the Holochain specific API functions as imports.
//#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
#[autotrace]
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
    let mut child = ht::with_top(|top| top.child(format!("{}:{}", file!(), line!())));
    child
        .as_mut()
        .map(|c| c.event(format!("{}:{}", file!(), line!())));

    // Create Imports with previously described Resolver
    let mut imports = ImportsBuilder::new();
    imports.push_resolver("env", &RuntimeModuleImportResolver);

    child
        .as_mut()
        .map(|c| c.event(format!("{}:{}", file!(), line!())));

    // Create module instance from wasm module, and start it if start is defined
    let m = ModuleInstance::new(&module, &imports).expect("Failed to instantiate module");
    child
        .as_mut()
        .map(|c| c.event(format!("{}:{}", file!(), line!())));
    let m = m
        .run_start(&mut NopExternals)
        .map_err(|_| HolochainError::RibosomeFailed("Module failed to start".to_string()));
    child
        .as_mut()
        .map(|c| c.event(format!("{}:{}", file!(), line!())));
    let _spanguard = child.map(|child| ht::push_span(child));
    m
}
