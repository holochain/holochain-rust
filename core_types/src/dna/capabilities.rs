//! File holding all the structs for handling capabilities

use crate::cas::content::Address;

//--------------------------------------------------------------------------------------------------
// CapabilityCall
//--------------------------------------------------------------------------------------------------
/// a struct to hold the signature of the call
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Hash)]
pub struct CallSignature {}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Hash)]
pub struct CapabilityCall {
    pub cap_token: Address,
    pub caller: Option<Address>,
    pub signature: CallSignature,
}

impl CapabilityCall {
    pub fn new(token: Address, caller: Option<Address>) -> Self {
        CapabilityCall {
            cap_token: token,
            caller,
            signature: CallSignature {}, // FIXME
        }
    }
}

//--------------------------------------------------------------------------------------------------
// CapabilityType
//--------------------------------------------------------------------------------------------------

/// Enum for Zome CapabilityType.  Public capabilities require public grant token.  Transferable
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
