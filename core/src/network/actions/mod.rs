pub mod custom_send;
pub mod get_entry;
pub mod get_links;
pub mod get_validation_package;
pub mod initialize_network;
pub mod publish;
pub mod shutdown;

use holochain_core_types::error::HcResult;
use holochain_persistence_api::cas::content::Address;

#[derive(Clone, Debug)]
pub enum ActionResponse {
    Publish(HcResult<Address>),
    Respond(HcResult<()>),
}
