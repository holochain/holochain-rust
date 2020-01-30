#[macro_use]
extern crate holochain_common;

pub mod conductor_api;
pub use conductor_api::ConductorApi;
pub use holochain_wasm_utils::api_serialization::crypto::CryptoMethod;

use lazy_static::lazy_static;

new_relic_setup!("NEW_RELIC_LICENSE_KEY");
