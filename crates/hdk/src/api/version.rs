use crate::{
    error::{ZomeApiError, ZomeApiResult},
};
use holochain_wasm_types::meta::{MetaArgs, MetaMethod, MetaResult};
use holochain_wasmer_guest::host_call;
use crate::api::hc_meta;

/// Returns the current `version` of the HDK as "semver" value (eg. "1.2.3-alpha4"), or
/// `version_hash`, a 32-byte MD5 of the holochain-rust source, dependencies and build environment,
/// such as "w7vyf4x77b1539rxakcqni8zdidpg7gy".  If the build environment is not Nix (and thus no
/// `out` or `HDK_HASH` environment variable is not supplied during build), a hash consisting of all
/// "0" is returned.
pub fn version() -> ZomeApiResult<String> {
    let meta = host_call!(hc_meta, MetaArgs {
        method: MetaMethod::Version,
    })?;
    let version = match meta {
        MetaResult::Version(ver) => Ok(ver),
        _ => Err(ZomeApiError::Internal(
            "Wrong MetaMethod/Result Type; Problem In Core".to_string(),
        )),
    }?;
    Ok(version)
}

pub fn version_hash() -> ZomeApiResult<String> {
    let meta = host_call!(hc_meta, MetaArgs {
        method: MetaMethod::Hash,
    })?;
    let hash = match meta {
        MetaResult::Hash(hash) => Ok(hash),
        _ => Err(ZomeApiError::Internal(
            "Wrong MetaMethod/Result Type; Problem In Core".to_string(),
        )),
    }?;
    Ok(hash)
}
