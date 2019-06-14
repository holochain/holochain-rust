use super::Dispatch;
use error::ZomeApiResult;
use holochain_json_api::json::JsonString;
use std::time::Duration;
/// Lets the DNA runtime sleep for the given duration.
/// # Examples
/// ```rust
/// # #[macro_use]
/// # extern crate hdk;
/// # use hdk::error::ZomeApiResult;
/// # use std::time::Duration;
///
/// # fn main() {
/// pub fn handle_some_function(content: String) -> ZomeApiResult<()> {
///     // ...
///     hdk::sleep(Duration::from_millis(100));
///     // ...
///     Ok(())
/// }
///
/// # }
/// ```
pub fn sleep(duration: Duration) -> ZomeApiResult<()> {
    let _: ZomeApiResult<()> = Dispatch::Sleep.with_input(JsonString::from(duration.as_nanos()));
    // internally returns RibosomeEncodedValue::Success which is a zero length allocation
    // return Ok(()) unconditionally instead of the "error" from success
    Ok(())
}
