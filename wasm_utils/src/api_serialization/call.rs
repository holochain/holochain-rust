use holochain_core_types::{error::HolochainError, json::*};
use std::convert::TryFrom;

/// Struct for input data received when Zome API function call() is invoked
#[derive(Deserialize, Default, Clone, PartialEq, Eq, Hash, Debug, Serialize)]
pub struct ZomeFnCallArgs {
    pub zome_name: String,
    pub cap_name: String,
    pub fn_name: String,
    pub fn_args: String,
}

impl From<ZomeFnCallArgs> for JsonString {
    fn from(v: ZomeFnCallArgs) -> Self {
        default_to_json(v)
    }
}

impl TryFrom<JsonString> for ZomeFnCallArgs {
    type Error = HolochainError;
    fn try_from(j: JsonString) -> Result<Self, Self::Error> {
        default_try_from_json(j)
    }
}
