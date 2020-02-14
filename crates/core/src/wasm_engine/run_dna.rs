use crate::{
    nucleus::ZomeFnResult,
    wasm_engine::{
        factories::{wasm_instance_factory, wasm_module_factory},
        memory::WasmPageManager,
        runtime::{Runtime, WasmCallData},
    },
    NEW_RELIC_LICENSE_KEY,
};
use holochain_core_types::error::{
    HcResult, HolochainError, RibosomeReturnValue, AllocationPtr,
};
use holochain_json_api::json::JsonString;

use crate::wasm_engine::factories::wasm_module_factory;
use holochain_wasm_utils::memory::{allocation::WasmAllocation, MemoryInt};
use std::convert::TryFrom;
use wasmer_runtime::{Module, Value};

/// Returns the WASM module, i.e. the WASM binary program code to run
/// for the given WasmCallData.
///
/// In case of a direct call, the module gets created from the WASM binary
/// inside the DirectCall specialisation for WasmCallData.
///
/// For ZomeCalls and CallbackCalls it gets the according module from the DNA.
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
fn get_module(data: WasmCallData) -> Result<ModuleArc, HolochainError> {
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
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn run_dna(parameters: Option<Vec<u8>>, data: WasmCallData) -> ZomeFnResult {
    let wasm_module = get_module(data.clone())?;

    // write input arguments for module call in memory Buffer
    let input_parameters: Vec<_> = parameters.unwrap_or_default();

    // instantiate runtime struct for passing external state data over wasm but not to wasm
    let mut wasm_instance = wasm_instance_factory(
        data.context()
            .map_err(|e| HolochainError::from(e.to_string()))?,
        &wasm_module,
    )?;

    let fn_name = data.fn_name();

    // scope for mutable borrow of runtime
    let host_allocation_ptr: AllocationPtr = {
        let maybe_allocation = runtime
            .memory_manager
            .write(&mut runtime.wasm_instance?, &input_parameters);

        match maybe_allocation {
            // No allocation to write is ok
            Err(AllocationError::ZeroLength) => RibosomeReturnValue::Success.into(),
            // Any other error is memory related
            Err(err) => {
                return Err(HolochainError::RibosomeFailed(format!(
                    "WASM Memory issue: {:?}. data = {:?}",
                    err, runtime.data
                )));
            }
            // Write successful, encode allocation
            Ok(allocation) => RibosomeReturnValue::from(allocation).into(),
        }
    };

    // for (byte, cell) in input_parameters
    //     .iter()
    //     .zip(
    //         wasm_instance.context_mut().memory(0).view()
    //             [0 as usize..(input_parameters.len()) as usize]
    //             .iter(),
    //     )
    // {
    //     cell.set(byte.to_owned())
    // }

    // scope for mutable borrow of runtime
    let returned_encoding = match {
        // Try installing a custom panic handler.
        // HDK-rust implements a function __install_panic_handler that reroutes output of
        // PanicInfo to hdk::debug.
        // Try calling it but fail silently if this function is not there.
        let _ = wasm_instance.call("__install_panic_handler", &[]);

        // invoke function in wasm instance
        // arguments are info for wasm on how to retrieve complex input arguments
        // which have been set in memory module
        wasm_instance
            .call(
                &fn_name,
                &[Value::I32(0), Value::I32(input_parameters.len() as _)],
            )
            .map_err(|err| {
                HolochainError::RibosomeFailed(format!(
                    "WASM invocation failed: {}. data = {:?}",
                    err, data
                ))
            })?
            .first()
            .ok_or_else(|| {
                HolochainError::RibosomeFailed(format!(
                    "WASM return value missing. data = {:?}",
                    data
                ))
            })?
            .to_owned()
    } {
        Value::I64(runtime_value) => runtime_value,
        _ => {
            return Err(HolochainError::RibosomeFailed(
                "WASM return value not I64".to_string(),
            ))
        }
    } as AllocationPtr;

    // Handle result returned by called zome function
    let return_code = RibosomeReturnValue::from(returned_encoding);

    let return_log_msg: String;
    let return_result: HcResult<JsonString>;

    match return_code.clone() {
        RibosomeReturnValue::Success => {
            return_log_msg = return_code.to_string();
            return_result = Ok(JsonString::null());
        }

        RibosomeReturnValue::Failure(err_code) => {
            return_log_msg = return_code.to_string();
            return_result = Err(HolochainError::RibosomeFailed(format!(
                "Zome function failure: {}",
                err_code.as_str()
            )));
            let log_message = format!(
                "err/nucleus/run_dna: Zome function failure: {}",
                err_code.as_str()
            );
            match &data {
                WasmCallData::ZomeCall(d) => {
                    log_info!(d.context, "{}, when calling: {:?}", log_message, d.call)
                }
                WasmCallData::CallbackCall(d) => {
                    log_info!(d.context, "{}, when calling: {:?}", log_message, d.call)
                }
                _ => {}
            };
        }

        RibosomeReturnValue::Allocation(ribosome_allocation) => {
            match WasmAllocation::try_from(ribosome_allocation) {
                Ok(allocation) => {
                    let memory = wasm_instance.context().memory(0);
                    let result: Vec<_> = memory.view()[MemoryInt::from(allocation.start()) as usize
                        ..MemoryInt::from(allocation.end()) as usize]
                        .iter()
                        .map(|cell| cell.get())
                        .collect();
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
