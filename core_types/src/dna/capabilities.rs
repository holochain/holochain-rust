//! File holding all the structs for handling capabilities

use crate::{
    cas::content::Address,
    signature::{test_signature, Signature},
};
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
pub struct CallSignature {
    signature: Signature,
}

impl CallSignature {
    pub fn new(signature: Signature) -> CallSignature {
        CallSignature { signature }
    }

    pub fn signature(&self) -> Signature {
        self.signature.clone()
    }
}

impl Default for CallSignature {
    fn default() -> CallSignature {
        CallSignature::new(test_signature())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Hash)]
pub struct CapabilityCall {
    pub cap_token: Address,
    pub caller: Address,
    pub signature: CallSignature,
}

impl CapabilityCall {
    pub fn new(token: Address, caller: Address, signature: CallSignature) -> Self {
        CapabilityCall {
            cap_token: token,
            caller,
            signature,
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
    fn test_capabilities_as_str() {
        assert_eq!(ReservedTraitNames::Public.as_str(), "hc_public");
    }

    #[test]
    fn test_capability_call_new() {
        let cap_call = CapabilityCall::new(
            Address::from("123"),
            Address::from("caller"),
            CallSignature::default(),
        );
        assert_eq!(
            CapabilityCall {
                cap_token: Address::from("123"),
                caller: Address::from("caller"),
                signature: CallSignature::default(),
            },
            cap_call
        );
    }

    #[test]
    fn test_call_signature_new() {
        let call_sig = CallSignature::new(test_signature());
        assert_eq!(call_sig.signature, test_signature());
    }

    #[test]
    fn test_call_signature_default() {
        let call_sig = CallSignature::default();
        assert_eq!(call_sig.signature, test_signature());
    }

}
