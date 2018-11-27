pub mod initialize_network;
pub mod publish;

use holochain_core_types::{cas::content::Address, error::HcResult};

#[derive(Clone, Debug)]
pub enum ActionResponse {
    Publish(HcResult<Address>),
    PublishLink(HcResult<Address>),
}
