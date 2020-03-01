use crate::{
    context::Context,
    nucleus::{CallbackFnCall, ZomeFnCall},
    NEW_RELIC_LICENSE_KEY,
};
use holochain_json_api::json::JsonString;
use holochain_core_types::error::HolochainError;
use std::{fmt, sync::Arc};
use wasmer_runtime::{error::RuntimeError, Instance, imports, func, instantiate};
use holochain_wasmer_host::WasmError;
use wasmer_runtime::Ctx;
use crate::workflows::debug::debug_workflow;
use holochain_wasm_types::ZomeApiResult;
use std::convert::TryInto;
use crate::workflows::get_links_count::get_link_result_count_workflow;

#[derive(Clone)]
pub struct ZomeCallData {
    /// Context of Holochain. Required for operating.
    pub context: Arc<Context>,
    /// The zome function call that initiated the Ribosome.
    pub call: ZomeFnCall,
}

#[derive(Clone)]
pub struct CallbackCallData {
    /// Context of Holochain. Required for operating.
    pub context: Arc<Context>,
    /// The callback function call that initiated the Ribosome.
    pub call: CallbackFnCall,
}

#[derive(Clone)]
pub enum WasmCallData {
    ZomeCall(ZomeCallData),
    CallbackCall(CallbackCallData),
    DirectCall(String, Arc<Vec<u8>>),
}

#[derive(Debug)]
struct BadCallError(String);
impl fmt::Display for BadCallError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Bad calling context: {}", self.0)
    }
}

// impl HostError for BadCallError {}

// #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
impl WasmCallData {
    pub fn new_zome_call(context: Arc<Context>, call: ZomeFnCall) -> Self {
        WasmCallData::ZomeCall(ZomeCallData { context, call })
    }

    pub fn new_callback_call(context: Arc<Context>, call: CallbackFnCall) -> Self {
        WasmCallData::CallbackCall(CallbackCallData { context, call })
    }

    pub fn fn_name(&self) -> String {
        match self {
            WasmCallData::ZomeCall(data) => data.call.fn_name.clone(),
            WasmCallData::CallbackCall(data) => data.call.fn_name.clone(),
            WasmCallData::DirectCall(name, _) => name.to_string(),
        }
    }

    pub fn context(&self) -> Result<Arc<Context>, HolochainError> {
        match &self {
            WasmCallData::ZomeCall(ref data) => Ok(data.context.clone()),
            WasmCallData::CallbackCall(ref data) => Ok(data.context.clone()),
            _ => Err(HolochainError::ErrorGeneric(format!("context data: {:?}", &self))),
        }
    }

    pub fn instance(&self) -> Result<Instance, HolochainError> {
        macro_rules! invoke_workflow {
            ( $workflow:ident ) => {{
                let closure_arc = std::sync::Arc::new(self.clone());
                move |ctx: &mut Ctx, guest_allocation_ptr: holochain_wasmer_host::AllocationPtr| -> ZomeApiResult {
                    let guest_bytes = holochain_wasmer_host::guest::read_from_allocation_ptr(ctx, guest_allocation_ptr)?;
                    let guest_json = JsonString::from(guest_bytes);
                    let context = std::sync::Arc::clone(&closure_arc.context().map_err(|_| WasmError::Unspecified )?);

                    Ok(holochain_wasmer_host::json::to_allocation_ptr(
                        context.block_on(
                            $workflow(context.clone(), guest_json.try_into()?)
                        ).map_err(|e| WasmError::Zome(e.to_string()))?.into()
                    ))
                }
            }}
        }

        let wasm_imports = imports! {
                "env" => {
                    "hc_debug" => func!(invoke_workflow!(debug_workflow)),
                    "hc_get_links_count" => func!(invoke_workflow!(get_link_result_count_workflow)),
                    // "hc_commit_entry" => zome_api_func!(context, invoke_commit_app_entry),
                },
            };

        let new_instance = |wasm: &Vec<u8>| {
            Ok(instantiate(wasm, &wasm_imports).map_err(|e| HolochainError::from(e.to_string()))?)
        };

        let (context, zome_name) = if let WasmCallData::DirectCall(_, wasm) = self {
            return new_instance(&wasm);
        } else {
            match self {
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

        new_instance(&wasm)
    }
}

impl fmt::Display for WasmCallData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WasmCallData::ZomeCall(data) => write!(f, "ZomeCall({:?})", data.call),
            WasmCallData::CallbackCall(data) => write!(f, "CallbackCall({:?})", data.call),
            WasmCallData::DirectCall(name, _) => write!(f, "DirectCall({})", name),
        }
    }
}

impl fmt::Debug for WasmCallData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "WasmCallData({})", self)
    }
}

/// Struct holding data of any call (callback or zome)
#[derive(Clone)]
pub struct CallData {
    pub context: Arc<Context>,
    pub zome_name: String,
    pub fn_name: String,
    pub parameters: JsonString,
}

/// Object holding data to pass around to invoked Zome API functions
// #[derive(Clone)]
pub struct Runtime {
    pub wasm_instance: Instance,

    /// data to be made available to the function at runtime
    pub data: WasmCallData,
}

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
impl Runtime {
    pub fn zome_call_data(&self) -> Result<ZomeCallData, RuntimeError> {
        match &self.data {
            WasmCallData::ZomeCall(ref data) => Ok(data.clone()),
            _ => Err(RuntimeError::Trap {
                msg: format!("zome_call_data: {:?}", &self.data).into_boxed_str(),
            }),
        }
    }

    pub fn callback_call_data(&self) -> Result<CallbackCallData, RuntimeError> {
        match &self.data {
            WasmCallData::CallbackCall(ref data) => Ok(data.clone()),
            _ => Err(RuntimeError::Trap {
                msg: format!("callback_call_data: {:?}", &self.data).into_boxed_str(),
            }),
        }
    }

    pub fn call_data(&self) -> Result<CallData, RuntimeError> {
        match &self.data {
            WasmCallData::ZomeCall(ref data) => Ok(CallData {
                context: data.context.clone(),
                zome_name: data.call.zome_name.clone(),
                fn_name: data.call.fn_name.clone(),
                parameters: data.call.parameters.clone(),
            }),
            WasmCallData::CallbackCall(ref data) => Ok(CallData {
                context: data.context.clone(),
                zome_name: data.call.zome_name.clone(),
                fn_name: data.call.fn_name.clone(),
                parameters: data.call.parameters.clone(),
            }),
            _ => Err(RuntimeError::Trap {
                msg: format!("call_data: {:?}", &self.data).into_boxed_str(),
            }),
        }
    }

    pub fn context(&self) -> Result<Arc<Context>, HolochainError> {
        self.data.context()
    }
}
