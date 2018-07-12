use error::HolochainError;
use net::SerialzedAddress;

#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    AddNode(SerializedAddress),
}
