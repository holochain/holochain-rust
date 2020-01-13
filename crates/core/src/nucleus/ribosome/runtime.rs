use crate::{
    context::Context,
    nucleus::{
        ribosome::{
            api::{ZomeApiFunction, ZomeApiResult},
            memory::WasmPageManager,
            Defn,
        },
        CallbackFnCall, ZomeFnCall,
    },
};
use holochain_core_types::error::{
    HolochainError, RibosomeEncodedValue, RibosomeEncodingBits, RibosomeRuntimeBits,
    ZomeApiInternalResult,
};

use holochain_json_api::json::JsonString;

use holochain_wasm_utils::memory::allocation::WasmAllocation;
use std::{convert::TryFrom, fmt, sync::Arc};
use wasmi::{Externals, HostError, RuntimeArgs, RuntimeValue, Trap, TrapKind};

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

impl HostError for BadCallError {}

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
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
#[derive(Clone)]
pub struct Runtime {
    /// Memory state tracker between ribosome and wasm.
    pub memory_manager: WasmPageManager,

    /// data to be made available to the function at runtime
    pub data: WasmCallData,
}

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
impl Runtime {
    pub fn zome_call_data(&self) -> Result<ZomeCallData, Trap> {
        match &self.data {
            WasmCallData::ZomeCall(ref data) => Ok(data.clone()),
            _ => Err(Trap::new(TrapKind::Host(Box::new(BadCallError(format!(
                "zome_call_data: {:?}",
                &self.data
            )))))),
        }
    }

    pub fn callback_call_data(&self) -> Result<CallbackCallData, Trap> {
        match &self.data {
            WasmCallData::CallbackCall(ref data) => Ok(data.clone()),
            _ => Err(Trap::new(TrapKind::Host(Box::new(BadCallError(format!(
                "callback_call_data: {:?}",
                &self.data
            )))))),
        }
    }

    pub fn call_data(&self) -> Result<CallData, Trap> {
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
            _ => Err(Trap::new(TrapKind::Host(Box::new(BadCallError(format!(
                "call_data: {:?}",
                &self.data
            )))))),
        }
    }

    pub fn context(&self) -> Result<Arc<Context>, Trap> {
        match &self.data {
            WasmCallData::ZomeCall(ref data) => Ok(data.context.clone()),
            WasmCallData::CallbackCall(ref data) => Ok(data.context.clone()),
            _ => Err(Trap::new(TrapKind::Host(Box::new(BadCallError(format!(
                "context data: {:?}",
                &self.data
            )))))),
        }
    }

    /// Load a JsonString stored in wasm memory.
    /// Input RuntimeArgs should only have one input which is the encoded allocation holding
    /// the complex data as an utf8 string.
    /// Returns the utf8 string.
    pub fn load_json_string_from_args(&self, args: &RuntimeArgs) -> JsonString {
        // @TODO don't panic in WASM
        // @see https://github.com/holochain/holochain-rust/issues/159
        assert_eq!(1, args.len());

        // Read complex argument serialized in memory
        let encoded: RibosomeEncodingBits = args.nth(0);
        let return_code = RibosomeEncodedValue::from(encoded);
        let allocation = match return_code {
            RibosomeEncodedValue::Success => return JsonString::null(),
            RibosomeEncodedValue::Failure(_) => {
                panic!("received error code instead of valid encoded allocation")
            }
            RibosomeEncodedValue::Allocation(ribosome_allocation) => {
                WasmAllocation::try_from(ribosome_allocation).unwrap()
            }
        };

        let bin_arg = self.memory_manager.read(allocation);

        // convert complex argument
        JsonString::from_json(
            &String::from_utf8(bin_arg)
                // @TODO don't panic in WASM
                // @see https://github.com/holochain/holochain-rust/issues/159
                .unwrap(),
        )
    }

    /// Store anything that implements Into<JsonString> in wasm memory.
    /// Note that From<T> for JsonString automatically implements Into
    /// Input should be a a json string.
    /// Returns a Result suitable to return directly from a zome API function, i.e. an encoded allocation
    pub fn store_as_json_string<J: Into<JsonString>>(&mut self, jsonable: J) -> ZomeApiResult {
        let j: JsonString = jsonable.into();
        // write str to runtime memory
        let mut s_bytes: Vec<_> = j.to_bytes();
        s_bytes.push(0); // Add string terminate character (important)

        match self.memory_manager.write(&s_bytes) {
            Err(_) => ribosome_error_code!(Unspecified),
            Ok(allocation) => Ok(Some(RuntimeValue::I64(RibosomeEncodingBits::from(
                RibosomeEncodedValue::Allocation(allocation.into()),
            )
                as RibosomeRuntimeBits))),
        }
    }

    pub fn store_result<J: Into<JsonString>>(
        &mut self,
        result: Result<J, HolochainError>,
    ) -> ZomeApiResult {
        self.store_as_json_string(match result {
            Ok(value) => ZomeApiInternalResult::success(value),
            Err(hc_err) => ZomeApiInternalResult::failure(core_error!(hc_err)),
        })
    }
}

// Correlate the indexes of core API functions with a call to the actual function
// by implementing the Externals trait from Wasmi.
impl Externals for Runtime {
    fn invoke_index(&mut self, index: usize, args: RuntimeArgs) -> ZomeApiResult {
        let zf = ZomeApiFunction::from_index(index);
        match zf {
            ZomeApiFunction::MissingNo => panic!("unknown function index"),
            // convert the function to its callable form and call it with the given arguments
            _ => zf.apply(self, &args),
        }
    }
}
