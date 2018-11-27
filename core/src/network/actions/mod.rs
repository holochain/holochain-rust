pub mod initialize_network;
pub mod publish;

use holochain_core_types::error::HcResult;
use holochain_core_types::cas::content::Address;

#[derive(Clone, Debug)]
pub enum ActionResponse {
    Publish(HcResult<Address>)
}
