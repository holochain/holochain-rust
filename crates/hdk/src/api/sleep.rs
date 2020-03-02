use crate::{error::ZomeApiResult};
use std::time::Duration;
use holochain_wasmer_guest::host_call;
use crate::api::hc_sleep;

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
    host_call!(hc_sleep, duration.as_nanos())?
}
