//! File holding all the structs for handling capabilities

use crate::cas::content::Address;

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
