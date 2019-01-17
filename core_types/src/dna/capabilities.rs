//! File holding all the structs for handling capabilities defined in DNA.

use crate::cas::content::Address;
use std::str::FromStr;

//--------------------------------------------------------------------------------------------------
// Reserved Capabilities names
//--------------------------------------------------------------------------------------------------

#[derive(Debug, PartialEq)]
/// Enumeration of all Capabilities known and used by HC Core
/// Enumeration converts to str
pub enum ReservedCapabilityNames {
    /// Development placeholder, no production fn should use MissingNo
    MissingNo,

    /// @TODO document what LifeCycle is
    /// @see https://github.com/holochain/holochain-rust/issues/204
    LifeCycle,

    /// @TODO document what Communication is
    /// @see https://github.com/holochain/holochain-rust/issues/204
    Communication,
}

impl FromStr for ReservedCapabilityNames {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "hc_lifecycle" => Ok(ReservedCapabilityNames::LifeCycle),
            "hc_web_gateway" => Ok(ReservedCapabilityNames::Communication),
            _ => Err("Cannot convert string to ReservedCapabilityNames"),
        }
    }
}

impl ReservedCapabilityNames {
    pub fn as_str(&self) -> &'static str {
        match *self {
            ReservedCapabilityNames::LifeCycle => "hc_lifecycle",
            ReservedCapabilityNames::Communication => "hc_web_gateway",
            ReservedCapabilityNames::MissingNo => "",
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
    pub cap_name: String,
    pub cap_token: Address,
    pub caller: Option<Address>,
    pub signature: CallSignature,
}

impl CapabilityCall {
    pub fn new(name: String, token: Address, caller: Option<Address>) -> Self {
        CapabilityCall {
            cap_name: name,
            cap_token: token,
            caller,
            signature: CallSignature {}, // FIXME
        }
    }
}

//--------------------------------------------------------------------------------------------------
// CapabilityType
//--------------------------------------------------------------------------------------------------

/// Enum for Zome CapabilityType.  Public capabilities require no token.  Transferable
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Hash)]
pub struct FnParameter {
    #[serde(rename = "type")]
    pub parameter_type: String,
    pub name: String,
}

impl FnParameter {
    #[allow(dead_code)]
    pub fn new<S: Into<String>>(n: S, t: S) -> FnParameter {
        FnParameter {
            name: n.into(),
            parameter_type: t.into(),
        }
    }
}

/// Represents a zome "fn_declarations" object.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Hash)]
pub struct FnDeclaration {
    /// The name of this fn declaration.
    #[serde(default)]
    pub name: String,
    pub inputs: Vec<FnParameter>,
    pub outputs: Vec<FnParameter>,
}

impl Default for FnDeclaration {
    /// Defaults for a "fn_declarations" object.
    fn default() -> Self {
        FnDeclaration {
            name: String::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
        }
    }
}

impl FnDeclaration {
    /// Allow sane defaults for `FnDecrlaration::new()`.
    pub fn new() -> Self {
        Default::default()
    }
}

/// Represents an individual object in the "zome" "capabilities" array.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Hash)]
pub struct Capability {
    /// capability type enum
    #[serde(rename = "type")]
    pub cap_type: CapabilityType,

    /// "functions" array
    #[serde(default)]
    pub functions: Vec<String>,
}

/// Represents an individual object in the "zome" "capabilities" array.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Hash)]
pub struct Aspect {
    /// capability type enum
    #[serde(rename = "type")]
    pub cap_type: CapabilityType,

    /// "functions" array
    #[serde(default)]
    pub functions: Vec<FnDeclaration>,
}

impl Default for Capability {
    /// Provide defaults for a Capability object
    fn default() -> Self {
        Capability {
            cap_type: CapabilityType::Assigned,
            functions: Vec::new(),
        }
    }
}

impl Capability {
    /// Capability Constructor
    pub fn new(cap_type: CapabilityType) -> Self {
        Capability {
            cap_type,
            functions: Vec::new(),
        }
    }
}

impl Aspect {
    /// Aspect Constructor
    pub fn new(cap_type: CapabilityType) -> Self {
        Aspect {
            cap_type,
            functions: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    /// test that a canonical string can be created from ReservedCapabilityNames
    fn test_capabilities_new() {
        let cap = Capability::default();
        assert_eq!(cap.cap_type, CapabilityType::Assigned);
        let cap = Capability::new(CapabilityType::Public);
        assert_eq!(cap.cap_type, CapabilityType::Public);
        let cap = Capability::new(CapabilityType::Transferable);
        assert_eq!(cap.cap_type, CapabilityType::Transferable);
    }

    #[test]
    /// test that ReservedCapabilityNames can be created from a canonical string
    fn test_capabilities_from_str() {
        assert_eq!(
            Ok(ReservedCapabilityNames::LifeCycle),
            ReservedCapabilityNames::from_str("hc_lifecycle"),
        );
        assert_eq!(
            Ok(ReservedCapabilityNames::Communication),
            ReservedCapabilityNames::from_str("hc_web_gateway"),
        );
        assert_eq!(
            Err("Cannot convert string to ReservedCapabilityNames"),
            ReservedCapabilityNames::from_str("foo"),
        );
    }

    #[test]
    /// test that a canonical string can be created from ReservedCapabilityNames
    fn test_capabilities_as_str() {
        assert_eq!(ReservedCapabilityNames::LifeCycle.as_str(), "hc_lifecycle");
        assert_eq!(
            ReservedCapabilityNames::Communication.as_str(),
            "hc_web_gateway",
        );
    }

    #[test]
    fn build_and_compare() {
        let fixture: Aspect = serde_json::from_str(
            r#"{
                "type": "transferable",
                "functions": [
                    {
                        "name": "test",
                        "inputs" : [
                            {
                                "name": "post",
                                "type": "string"
                            }
                        ],
                        "outputs" : [
                            {
                                "name": "hash",
                                "type": "string"
                            }
                        ]
                    }
                ]
            }"#,
        )
        .unwrap();

        let mut cap = Aspect::new(CapabilityType::Transferable);
        let mut fn_dec = FnDeclaration::new();
        fn_dec.name = String::from("test");
        let input = FnParameter::new("post", "string");
        let output = FnParameter::new("hash", "string");
        fn_dec.inputs.push(input);
        fn_dec.outputs.push(output);
        cap.functions.push(fn_dec);

        assert_eq!(fixture, cap);
    }
}
