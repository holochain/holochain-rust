//! File holding all the structs for handling capabilities defined in DNA.

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
// CapabilityType
//--------------------------------------------------------------------------------------------------

/// Enum for Zome Capability "membrane" property.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Hash)]
pub enum Membrane {
    #[serde(rename = "public")]
    Public,
    #[serde(rename = "agent")]
    Agent,
    #[serde(rename = "api-key")]
    ApiKey,
    #[serde(rename = "zome")]
    Zome,
}

impl Default for Membrane {
    /// Default zome capability membrane is "agent"
    fn default() -> Self {
        Membrane::Agent
    }
}

/// Represents the "capability" sub-object on a "zome" "capabilities" object.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Hash)]
pub struct CapabilityType {
    /// How visibility should be handled for this capability.
    #[serde(default)]
    pub membrane: Membrane,
}

impl Default for CapabilityType {
    /// Defaults for a "capability" sub-object on a "zome" "capabilities" object.
    fn default() -> Self {
        CapabilityType {
            membrane: Membrane::Agent,
        }
    }
}

impl CapabilityType {
    /// Allow sane defaults for `CapabilityType::new()`.
    pub fn new() -> Self {
        Default::default()
    }
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
    /// "capability" sub-object
    #[serde(rename = "capability")]
    pub cap_type: CapabilityType,

    /// "fn_declarations" array
    #[serde(default)]
    pub functions: Vec<FnDeclaration>,
}

impl Default for Capability {
    /// Provide defaults for a "zome"s "capabilities" object.
    fn default() -> Self {
        Capability {
            cap_type: CapabilityType::new(),
            functions: Vec::new(),
        }
    }
}

impl Capability {
    /// Allow sane defaults for `Capability::new()`.
    pub fn new() -> Self {
        Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

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
        let fixture: Capability = serde_json::from_str(
            r#"{
                "capability": {
                    "membrane": "agent"
                },
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
        ).unwrap();

        let mut cap = Capability::new();
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
