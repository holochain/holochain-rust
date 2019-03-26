use boolinator::Boolinator;
use holochain_core_types::{
    agent::AgentId, cas::content::AddressableContent, error::HolochainError,
};

pub fn request_signing_service(
    agent_id: &AgentId,
    payload: &String,
    signing_service_uri: &String,
) -> Result<String, HolochainError> {
    let body_json = json!({"agent_id": agent_id.address(), "payload": payload});
    let client = reqwest::Client::new();
    let url = reqwest::Url::parse(signing_service_uri).map_err(|_| {
        HolochainError::ConfigError(format!(
            "Can't parse signing service URI: '{}'",
            signing_service_uri
        ))
    })?;
    // NB: .json sets content-type: application/json
    let mut response = client.post(url).json(&body_json).send().map_err(|e| {
        HolochainError::ErrorGeneric(format!("Error during signing request: {:?}", e))
    })?;
    response
        .status()
        .is_success()
        .ok_or(HolochainError::new(&format!(
            "Status of response from signing service is not success: {:#?}",
            response
        )))?;
    response
        .text()
        .map_err(|_| HolochainError::new("Signing service response has no text"))
}
