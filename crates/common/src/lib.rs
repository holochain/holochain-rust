extern crate holochain_json_api;
#[macro_use]
extern crate holochain_json_derive;
#[macro_use]
extern crate serde_derive;

pub mod env_vars;
pub mod paths;

// TODO: Remove this as soon as we have keystores that can store and lock multiple keys with a single passphrase.
// (This is just for bootstrapping while still in alpha)
pub const DEFAULT_PASSPHRASE: &str = "convenient and insecure keystore";

use holochain_json_api::{error::JsonError, json::JsonString};

#[derive(Deserialize, Serialize, Clone, Debug, DefaultJson, PartialEq)]
pub struct FakeSim1hConfig {
    pub dynamo_url: String,
}
