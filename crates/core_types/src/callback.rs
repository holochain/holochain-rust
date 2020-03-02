use holochain_json_api::{error::JsonError, json::JsonString};
use validation::ValidationPackageDefinition;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, DefaultJson)]
pub enum CallbackResult {
    Pass,
    Fail(String),
    NotImplemented(String),
    ValidationPackageDefinition(ValidationPackageDefinition),
    ReceiveResult(String),
}
