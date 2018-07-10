//! holochain_dna::zome::capabilities is a set of structs for working with holochain dna.

use std::str::FromStr;
use wasm::DnaWasm;

//--------------------------------------------------------------------------------------------------
// Reserved Capabilities and functions names
//--------------------------------------------------------------------------------------------------

/// Enumeration of all Capabilities known and used by HC Core
/// Enumeration converts to str
pub enum ReservedCapabilityNames {
    LifeCycle,
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
        }
    }
}

/// Enumeration of all Zome functions known and used by HC Core
/// Enumeration converts to str
pub enum ReservedFunctionNames {
    /// genesis() -> bool
    /// Must be in LifeCycle Capability
    Genesis,
    /// receive(from : String, message : String) -> String
    /// Must be in Communication Capability
    Receive,
}

impl FromStr for ReservedFunctionNames {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "genesis" => Ok(ReservedFunctionNames::Genesis),
            "receive" => Ok(ReservedFunctionNames::Receive),
            _ => Err("Cannot convert string to ReservedFunctionNames"),
        }
    }
}

impl ReservedFunctionNames {
    pub fn as_str(&self) -> &'static str {
        match *self {
            ReservedFunctionNames::Genesis => "genesis",
            ReservedFunctionNames::Receive => "receive",
        }
    }
}

//--------------------------------------------------------------------------------------------------
//
//--------------------------------------------------------------------------------------------------

/// Enum for Zome Capability "membrane" property.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
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
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
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

/// Represents a zome "fn_declarations" object.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FnDeclaration {
    /// The name of this fn declaration.
    #[serde(default)]
    pub name: String,
    // TODO - signature
}

impl Default for FnDeclaration {
    /// Defaults for a "fn_declarations" object.
    fn default() -> Self {
        FnDeclaration {
            name: String::from(""),
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
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Capability {
    /// The name of this capability.
    #[serde(default)]
    pub name: String,

    /// "capability" sub-object
    #[serde(default)]
    pub capability: CapabilityType,

    /// "fn_declarations" array
    #[serde(default)]
    pub fn_declarations: Vec<FnDeclaration>,

    /// Validation code for this entry_type.
    #[serde(default)]
    pub code: DnaWasm,
}

impl Default for Capability {
    /// Provide defaults for a "zome"s "capabilities" object.
    fn default() -> Self {
        Capability {
            name: String::from(""),
            capability: CapabilityType::new(),
            fn_declarations: Vec::new(),
            code: DnaWasm::new(),
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
    fn build_and_compare() {
        let fixture: Capability = serde_json::from_str(
            r#"{
                "name": "test",
                "capability": {
                    "membrane": "agent"
                },
                "fn_declarations": [
                    {
                        "name": "test"
                    }
                ],
                "code": {
                    "code": "AAECAw=="
                }
            }"#,
        ).unwrap();

        let mut cap = Capability::new();
        cap.name = String::from("test");
        let mut fn_dec = FnDeclaration::new();
        fn_dec.name = String::from("test");
        cap.fn_declarations.push(fn_dec);
        cap.code.code = vec![0, 1, 2, 3];

        assert_eq!(fixture, cap);
    }
}
