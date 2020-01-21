use crate::{
    nucleus::ZomeFnResult,
    wasm_engine::{
        factories::{wasm_instance_factory, wasm_module_factory},
        memory::WasmPageManager,
        runtime::{Runtime, WasmCallData},
    },
};
use holochain_core_types::{
    dna::wasm::ModuleArc,
    error::{
        HcResult, HolochainError, RibosomeEncodedValue, RibosomeEncodingBits, RibosomeRuntimeBits,
    },
};
use holochain_json_api::json::JsonString;

use holochain_wasm_utils::memory::allocation::{AllocationError, WasmAllocation};
use std::convert::TryFrom;
use wasmi::RuntimeValue;

/// Returns the WASM module, i.e. the WASM binary program code to run
/// for the given WasmCallData.
///
/// In case of a direct call, the module gets created from the WASM binary
/// inside the DirectCall specialisation for WasmCallData.
///
/// For ZomeCalls and CallbackCalls it gets the according module from the DNA.
fn get_module(data: WasmCallData) -> Result<ModuleArc, HolochainError> {
    let (context, zome_name) = if let WasmCallData::DirectCall(_, wasm) = data {
        let transient_module = ModuleArc::new(wasm_module_factory(wasm)?);
        return Ok(transient_module);
    } else {
        match data {
            WasmCallData::ZomeCall(d) => (d.context.clone(), d.call.zome_name),
            WasmCallData::CallbackCall(d) => (d.context.clone(), d.call.zome_name),
            WasmCallData::DirectCall(_, _) => unreachable!(),
        }
    };

    let state_lock = context.state()?;
    let module = state_lock
        .nucleus()
        .dna
        .as_ref()
        .unwrap()
        .zomes
        .get(&zome_name)
        .ok_or_else(|| HolochainError::new(&format!("No Ribosome found for Zome '{}'", zome_name)))?
        .code
        .get_wasm_module()?;

    Ok(module)
}

/// Executes an exposed zome function in a wasm binary.
/// Multithreaded function
/// panics if wasm binary isn't valid.
pub fn run_dna(parameters: Option<Vec<u8>>, data: WasmCallData) -> ZomeFnResult {
    let wasm_module = get_module(data.clone())?;
    let wasm_instance = wasm_instance_factory(&wasm_module)?;
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
                    "WASM Memory issue: {:?}. data = {:?}",
                    err, runtime.data
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
                    encoded_allocation_of_input as RibosomeRuntimeBits,
                )],
                mut_runtime,
            )
            .map_err(|err| {
                HolochainError::RibosomeFailed(format!(
                    "WASM invocation failed: {}. data = {:?}",
                    err, runtime.data
                ))
            })?
            .unwrap()
            .try_into() // Option<_>
            .ok_or_else(|| {
                HolochainError::RibosomeFailed(format!(
                    "WASM return value missing. data = {:?}",
                    runtime.data
                ))
            })?
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
            let log_message = format!(
                "err/nucleus/run_dna: Zome function failure: {}",
                err_code.as_str()
            );
            match &runtime.data {
                WasmCallData::ZomeCall(d) => {
                    log_info!(d.context, "{}, when calling: {:?}", log_message, d.call)
                }
                WasmCallData::CallbackCall(d) => {
                    log_info!(d.context, "{}, when calling: {:?}", log_message, d.call)
                }
                _ => {}
            };
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
    // runtime. log_debug!(context, "zome: Zome Function '{}' returned: {}",
    //     zome_call.fn_name, return_log_msg,
    // );
    let _ = return_log_msg;
    return_result
}
