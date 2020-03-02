// pub mod callback;
pub mod runtime;
pub use self::{runtime::*};
pub mod callback;
pub mod io;
pub use holochain_wasmer_host::*;

pub const MAX_ZOME_CALLS: usize = 10;

#[cfg(test)]
pub mod tests {

    // use crate::wasm_engine::runtime::WasmCallData;
    // use crate::nucleus::tests::test_capability_request;
    // use crate::context::Context;
    // use std::sync::Arc;
    use holochain_wasm_types::JsonString;

    /// dummy zome name
    pub fn test_zome_name() -> String {
        "test_zome".to_string()
    }

    /// dummy zome API function name
    pub fn test_function_name() -> String {
        "test".to_string()
    }

    /// dummy parameters for a zome API function call
    pub fn test_parameters() -> JsonString {
        JsonString::empty_object()
    }

    // pub fn test_wasm_call_data(context: Arc<Context>) -> WasmCallData {
    //     let zome_call = ZomeFnCall::new(
    //         &test_zome_name(),
    //         test_capability_request(context.clone(), &test_function_name(), test_parameters()),
    //         &test_function_name(),
    //         test_parameters(),
    //     );
    //
    //     WasmCallData::new_zome_call(context, zome_call)
    // }

}
