use api::BundleOnClose;
use error::{ZomeApiError, ZomeApiResult};

/// NOT YET AVAILABLE
pub fn start_bundle(_timeout: usize, _user_param: serde_json::Value) -> ZomeApiResult<()> {
    Err(ZomeApiError::FunctionNotImplemented)
}

/// NOT YET AVAILABLE
pub fn close_bundle(_action: BundleOnClose) -> ZomeApiResult<()> {
    Err(ZomeApiError::FunctionNotImplemented)
}
