use crate::{
    context::Context,
    nucleus::{CallbackFnCall, ZomeFnCall},
    NEW_RELIC_LICENSE_KEY,
};
use holochain_json_api::json::JsonString;

use std::{fmt, sync::Arc};
use wasmer_runtime::{error::RuntimeError, Instance};

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

    pub fn context(&self) -> Result<Arc<Context>, RuntimeError> {
        match &self {
            WasmCallData::ZomeCall(ref data) => Ok(data.context.clone()),
            WasmCallData::CallbackCall(ref data) => Ok(data.context.clone()),
            _ => Err(RuntimeError::Trap {
                msg: format!("context data: {:?}", &self).into_boxed_str(),
            }),
        }
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

// #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
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

    pub fn context(&self) -> Result<Arc<Context>, RuntimeError> {
        self.data.context()
    }
}
