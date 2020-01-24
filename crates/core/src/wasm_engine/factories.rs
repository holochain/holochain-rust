use holochain_core_types::error::HolochainError;
use std::sync::Arc;
use wasmer_runtime::{imports, instantiate, Instance, Module};

/// Creates a WASM module, that is the executable program, from a given WASM binary byte array.
pub fn wasm_module_factory(wasm: Arc<Vec<u8>>) -> Result<Module, HolochainError> {
    let import_object = imports! {};
    Ok(instantiate(&wasm, &import_object)
        .map_err(|e| HolochainError::from(e.to_string()))?
        .module())
}

/// Creates a runnable WASM module instance from a module reference.
/// Adds the Holochain specific API functions as imports.
pub fn wasm_instance_factory(module: &Module) -> Result<Instance, HolochainError> {
    let import_object = imports! {};
    Ok(module
        .instantiate(&import_object)
        .map_err(|e| HolochainError::from(e.to_string()))?)
}
