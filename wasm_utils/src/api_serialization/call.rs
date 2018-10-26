use holochain_core_types::json::default_to_json_string;

/// Struct for input data received when Zome API function call() is invoked
#[derive(Deserialize, Default, Clone, PartialEq, Eq, Hash, Debug, Serialize)]
pub struct ZomeFnCallArgs {
    pub zome_name: String,
    pub cap_name: String,
    pub fn_name: String,
    pub fn_args: String,
}

impl From<ZomeFnCallArgs> for JsonString {
    fn from(zome_fn_call_args: ZomeFnCallArgs) -> JsonString {
        default_to_json_string(zome_fn_call_args)
    }
}
