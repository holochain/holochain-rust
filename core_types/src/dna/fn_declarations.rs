//! File holding all the structs for handling function declarations defined in DNA.

use crate::dna::capabilities::CapabilityType;

/// Represents the type declaration for zome function parameter
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

/// Represents a zome function declaration
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

/// Represents an trait definition for bridging
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Hash)]
pub struct Trait {
    /// capability type enum
    #[serde(rename = "type")]
    pub cap_type: CapabilityType,

    /// "functions" array
    #[serde(default)]
    pub functions: Vec<FnDeclaration>,
}

impl Trait {
    /// Trait Constructor
    pub fn new(cap_type: CapabilityType) -> Self {
        Trait {
            cap_type,
            functions: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dna::capabilities::CapabilityType;

    #[test]
    fn test_trait_build_and_compare() {
        let fixture: Trait = serde_json::from_str(
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

        let mut trt = Trait::new(CapabilityType::Transferable);
        let mut fn_dec = FnDeclaration::new();
        fn_dec.name = String::from("test");
        let input = FnParameter::new("post", "string");
        let output = FnParameter::new("hash", "string");
        fn_dec.inputs.push(input);
        fn_dec.outputs.push(output);
        trt.functions.push(fn_dec);

        assert_eq!(fixture, trt);
    }
}
