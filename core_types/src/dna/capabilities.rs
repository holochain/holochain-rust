//! File holding all the structs for handling capabilities

use crate::cas::content::Address;
use std::str::FromStr;

//--------------------------------------------------------------------------------------------------
// Reserved Trait names
//--------------------------------------------------------------------------------------------------

#[derive(Debug, PartialEq)]
/// Enumeration of all Traits known and used by HC Core
/// Enumeration converts to str
pub enum ReservedTraitNames {
    /// Development placeholder, no production fn should use MissingNo
    MissingNo,

    /// used for declaring functions that will auto-generate a public grant during genesis
    Public,
}

impl FromStr for ReservedTraitNames {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "hc_public" => Ok(ReservedTraitNames::Public),
            _ => Err("Cannot convert string to ReservedTraitNames"),
        }
    }
}

impl ReservedTraitNames {
    pub fn as_str(&self) -> &'static str {
        match *self {
            ReservedTraitNames::Public => "hc_public",
            ReservedTraitNames::MissingNo => "",
        }
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// test that ReservedTraitNames can be created from a canonical string
    fn test_capabilities_from_str() {
        assert_eq!(
            Ok(ReservedTraitNames::Public),
            ReservedTraitNames::from_str("hc_public"),
        );
        assert_eq!(
            Err("Cannot convert string to ReservedTraitNames"),
            ReservedTraitNames::from_str("foo"),
        );
    }

    #[test]
    /// test that a canonical string can be created from ReservedTraitNames
    fn test_reserved_traits_as_str() {
        assert_eq!(ReservedTraitNames::Public.as_str(), "hc_public");
    }

}
