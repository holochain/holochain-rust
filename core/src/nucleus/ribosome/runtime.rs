use crate::{
    context::Context,
    nucleus::{
        ribosome::{
            api::{ZomeApiFunction, ZomeApiResult},
            memory::SinglePageManager,
            Defn,
            fn_call::ZomeFnCall,
        },
    },
};
use holochain_core_types::{
    error::{HolochainError, RibosomeReturnCode, ZomeApiInternalResult},
    json::JsonString,
};
use holochain_wasm_utils::memory_allocation::decode_encoded_allocation;
use std::sync::Arc;
use wasmi::{Externals, RuntimeArgs, RuntimeValue};

/// Object holding data to pass around to invoked Zome API functions
#[derive(Clone)]
pub struct Runtime {
    /// Memory state tracker between ribosome and wasm.
    pub memory_manager: SinglePageManager,
    /// Context of Holochain. Required for operating.
    pub context: Arc<Context>,
    /// Name of the DNA that is being hosted.
    pub dna_name: String,
    /// The zome function call that initiated the Ribosome.
    pub zome_call: ZomeFnCall,
}

impl Runtime {
    /// Load a JsonString stored in wasm memory.
    /// Input RuntimeArgs should only have one input which is the encoded allocation holding
    /// the complex data as an utf8 string.
    /// Returns the utf8 string.
    pub fn load_json_string_from_args(&self, args: &RuntimeArgs) -> JsonString {
        // @TODO don't panic in WASM
        // @see https://github.com/holochain/holochain-rust/issues/159
        assert_eq!(1, args.len());

        // Read complex argument serialized in memory
        let encoded_allocation: u32 = args.nth(0);
        let maybe_allocation = decode_encoded_allocation(encoded_allocation);
        let allocation = match maybe_allocation {
            // Handle empty allocation edge case
            Err(RibosomeReturnCode::Success) => return JsonString::null(),
            // Handle error code
            Err(_) => panic!("received error code instead of valid encoded allocation"),
            // Handle normal allocation
            Ok(allocation) => allocation,
        };
        let bin_arg = self.memory_manager.read(allocation);

        // convert complex argument
        String::from_utf8(bin_arg)
            // @TODO don't panic in WASM
            // @see https://github.com/holochain/holochain-rust/issues/159
            .unwrap()
            .into()
    }

    /// Store anything that implements Into<JsonString> in wasm memory.
    /// Note that From<T> for JsonString automatically implements Into
    /// Input should be a a json string.
    /// Returns a Result suitable to return directly from a zome API function, i.e. an encoded allocation
    pub fn store_as_json_string<J: Into<JsonString>>(&mut self, jsonable: J) -> ZomeApiResult {
        let j: JsonString = jsonable.into();
        // write str to runtime memory
        let mut s_bytes: Vec<_> = j.into_bytes();
        s_bytes.push(0); // Add string terminate character (important)

        match self.memory_manager.write(&s_bytes) {
            Err(_) => ribosome_error_code!(Unspecified),
            Ok(allocation) => Ok(Some(RuntimeValue::I32(allocation.encode() as i32))),
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
            _ => zf.as_fn()(self, &args),
        }
    }
}
