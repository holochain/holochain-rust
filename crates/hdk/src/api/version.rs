use crate::{error::ZomeApiResult, Dispatch};
use holochain_wasm_utils::api_serialization::meta::{MetaArgs, MetaMethod, MetaResult};

///this method will return the current version of the HDK
pub fn version<S: Into<String>>() -> ZomeApiResult<String> {
    let meta = Dispatch::Meta.with_input(MetaArgs {
        method: MetaMethod::Version,
    })?;
    let version: String = match meta {
        MetaResult::Version(ver) => ver.to_string(),
    };

    Ok(version)
}
