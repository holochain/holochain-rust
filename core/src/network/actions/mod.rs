pub mod custom_send;
pub mod get_entry;
pub mod get_links;
pub mod get_validation_package;
pub mod initialize_network;
pub mod publish;

use holochain_core_types::{cas::content::Address, error::HcResult};

#[derive(Clone, Debug)]
pub enum ActionResponse {
    Publish(HcResult<Address>),
    RespondGet(HcResult<()>),
    RespondGetLinks(HcResult<()>),
}
