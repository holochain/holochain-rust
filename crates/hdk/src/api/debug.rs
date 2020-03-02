use crate::{error::ZomeApiResult};
use holochain_wasmer_guest::*;
use holochain_wasm_types::wasm_string::WasmString;
use crate::api::hc_debug;

/// Prints a string through the stdout of the running Conductor, and also
/// writes that string to the logger in the execution context
/// # Examples
/// ```rust
/// # #[macro_use]
/// # extern crate hdk;
/// # use hdk::error::ZomeApiResult;
///
/// # fn main() {
/// pub fn handle_some_function(content: String) -> ZomeApiResult<()> {
///     // ...
///     hdk::debug("write a message to the logs");
///     // ...
///     Ok(())
/// }
///
/// # }
/// ```
pub fn debug<J: Into<String>>(msg: J) -> ZomeApiResult<()> {
    let s: String = msg.into();
    let ws = WasmString::from(s);

    Ok(host_call!(hc_debug, ws)?)
}
