/// capabilities implements the capability request functionality used to check
/// that a given capability has been granted for actions like zome calls
use crate::{
    cas::content::Address,
    error::HolochainError,
    json::JsonString,
    signature::{Provenance, Signature},
};

//--------------------------------------------------------------------------------------------------
// CapabilityType
//--------------------------------------------------------------------------------------------------

/// Enum for CapabilityType.  Public capabilities require public grant token.  Transferable
/// capabilities require a token, but don't limit the capability to specific agent(s);
/// this functions like a password in that you can give the token to someone else and it works.
/// Assigned capabilities check the request's signature against the list of agents to which
/// the capability has been granted.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Hash)]
pub enum CapabilityType {
    #[serde(rename = "public")]
    Public,
    #[serde(rename = "transferable")]
    Transferable,
    #[serde(rename = "assigned")]
    Assigned,
}

//--------------------------------------------------------------------------------------------------
// CapabilityRequest
//--------------------------------------------------------------------------------------------------

/// a struct to hold the capability information needed to make any capability request,
/// namely the provenance of the request (the agent address an signature) and the
/// actual token being used to make the request
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Hash, DefaultJson)]
pub struct CapabilityRequest {
    pub cap_token: Address,
    pub provenance: Provenance,
}

impl CapabilityRequest {
    pub fn new(token: Address, requester: Address, signature: Signature) -> Self {
        CapabilityRequest {
            cap_token: token,
            provenance: Provenance::new(requester, signature),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::cas::content::Address;

    #[test]
    fn test_capability_request_new() {
        let cap_call = CapabilityRequest::new(
            Address::from("123"),
            Address::from("requester"),
            Signature::fake(),
        );
        assert_eq!(
            CapabilityRequest {
                cap_token: Address::from("123"),
                provenance: Provenance::new(Address::from("requester"), Signature::fake()),
            },
            cap_call
        );
    }
}
