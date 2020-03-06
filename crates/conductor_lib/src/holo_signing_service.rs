use boolinator::Boolinator;
use holochain_core_types::{agent::AgentId, error::HolochainError};
use holochain_persistence_api::cas::content::AddressableContent;

// this could be used for a lot of external callbacks and can be moved to somewhere more general?
//#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CONDUCTOR_LIB)]
pub fn request_service(
    agent_id: &AgentId,
    payload: &String,
    service_uri: &String,
) -> Result<String, HolochainError> {
    let body_json = json!({"agent_id": agent_id.address(), "payload": payload});
    let client = reqwest::Client::new();
    let url = reqwest::Url::parse(service_uri).map_err(|_| {
        HolochainError::ConfigError(format!("Can't parse service URI: '{}'", service_uri))
    })?;
    // NB: .json sets content-type: application/json
    let mut response = client
        .post(url)
        .json(&body_json)
        .send()
        .map_err(|e| HolochainError::ErrorGeneric(format!("Error during request: {:?}", e)))?;
    response
        .status()
        .is_success()
        .ok_or(HolochainError::new(&format!(
            "Status of response from service is not success: {:#?}",
            response
        )))?;
    response
        .text()
        .map_err(|_| HolochainError::new("Service response has no text"))
}
