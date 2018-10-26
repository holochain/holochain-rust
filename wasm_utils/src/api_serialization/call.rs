use holochain_core_types::{convert::TryFrom, error::HolochainError, json::*};

/// Struct for input data received when Zome API function call() is invoked
#[derive(Deserialize, Default, Clone, PartialEq, Eq, Hash, Debug, Serialize)]
pub struct ZomeFnCallArgs {
    pub zome_name: String,
    pub cap_name: String,
    pub fn_name: String,
    pub fn_args: String,
}

impl TryFrom<ZomeFnCallArgs> for JsonString {
    type Error = HolochainError;
    fn try_from(v: ZomeFnCallArgs) -> JsonResult {
        default_try_to_json(v)
    }
}
