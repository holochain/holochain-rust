pub mod custom_send;
pub mod get_validation_package;
pub mod initialize_network;
pub mod publish;
pub mod publish_header_entry;
pub mod query;
pub mod shutdown;

use crate::state::ActionResponse;
use holochain_core_types::error::HcResult;
use holochain_json_api::{error::JsonError, json::JsonString};
use holochain_persistence_api::cas::content::Address;
use std::ops::Deref;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, DefaultJson)]
pub enum NetworkActionResponse {
    Publish(HcResult<Address>),
    PublishHeaderEntry(HcResult<Address>),
    Respond(HcResult<()>),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, DefaultJson)]
pub struct Response(ActionResponse<NetworkActionResponse>);

impl Deref for Response {
    type Target = ActionResponse<NetworkActionResponse>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<NetworkActionResponse> for Response {
    fn from(r: NetworkActionResponse) -> Self {
        Response(ActionResponse::new(r))
    }
}
