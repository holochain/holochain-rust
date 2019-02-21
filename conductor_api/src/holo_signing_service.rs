use holochain_core_types::{
    agent::AgentId,
    cas::content::AddressableContent,
    error::HolochainError
};

pub fn request_signing_service(
    agent_id: &AgentId,
    payload: &String,
    signing_service_uri: &String
) -> Result<String, HolochainError> {
    let body_json = json!({"agent_id": agent_id.address(), "payload": payload});
    let body = serde_json::to_string(&body_json).unwrap();
    let client = reqwest::Client::new();
    let url = reqwest::Url::parse(signing_service_uri)
        .map_err(|_| HolochainError::new("Can't parse signing service URI"))?;
    let mut response = client.post(url)
        .body(body)
        .send()
        .map_err(|e|
            HolochainError::ErrorGeneric(format!("Error during signing request: {:?}", e))
        )?;
    response.text().map_err(|_| HolochainError::new("Signing service response has no text"))
}