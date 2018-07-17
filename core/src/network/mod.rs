//use error::HolochainError;
use holochain_net::SerializedAddress;

#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    AddNode(SerializedAddress),
}
