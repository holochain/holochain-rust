//! holochain_dna::zome is a set of structs for working with holochain dna.

pub mod capabilities;
pub mod entry_types;

use std::collections::HashMap;
use wasm::DnaWasm;

/// Enum for "zome" "config" "error_handling" property.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Hash)]
pub enum ErrorHandling {
    #[serde(rename = "throw-errors")]
    ThrowErrors,
}

impl Default for ErrorHandling {
    /// Default zome config error_handling is "throw-errors"
    fn default() -> Self {
        ErrorHandling::ThrowErrors
    }
}

/// Represents the "config" object on a "zome".
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Hash)]
pub struct Config {
    /// How errors should be handled within this zome.
    #[serde(default)]
    pub error_handling: ErrorHandling,
}

impl Default for Config {
    /// Provide defaults for the "zome" "config" object.
    fn default() -> Self {
        Config {
            error_handling: ErrorHandling::ThrowErrors,
        }
    }
}

impl Config {
    /// Allow sane defaults for `Config::new()`.
    pub fn new() -> Self {
        Default::default()
    }
}

/// Represents an individual "zome".
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Zome {
    /// A description of this zome.
    #[serde(default)]
    pub description: String,

    /// Configuration associated with this zome.
    /// Note, this should perhaps be a more free-form serde_json::Value,
    /// "throw-errors" may not make sense for wasm, or other ribosome types.
    #[serde(default)]
    pub config: Config,

    /// An array of entry_types associated with this zome.
    #[serde(default)]
    pub entry_types: HashMap<String, entry_types::EntryTypeDef>,

    /// An array of capabilities associated with this zome.
    #[serde(default)]
    pub capabilities: HashMap<String, capabilities::Capability>,

    /// Validation code for this entry_type.
    #[serde(default)]
    pub code: DnaWasm,
}

impl Eq for Zome {}

impl Default for Zome {
    /// Provide defaults for an individual "zome".
    fn default() -> Self {
        Zome {
            description: String::new(),
            config: Config::new(),
            entry_types: HashMap::new(),
            capabilities: HashMap::new(),
            code: DnaWasm::new(),
        }
    }
}

impl Zome {
    /// Allow sane defaults for `Zome::new()`.
    pub fn new(
        description: &str,
        config: &Config,
        entry_types: &HashMap<String, entry_types::EntryTypeDef>,
        capabilities: &HashMap<String, capabilities::Capability>,
        code: &DnaWasm,
    ) -> Zome {
        Zome {
            description: description.into(),
            config: config.clone(),
            entry_types: entry_types.to_owned(),
            capabilities: capabilities.to_owned(),
            code: code.clone(),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn build_and_compare() {
        let fixture: Zome = serde_json::from_str(
            r#"{
                "description": "test",
                "config": {
                    "error_handling": "throw-errors"
                },
                "entry_types": {},
                "capabilities": {},
                "code": {
                    "code": "AAECAw=="
                }
            }"#,
        ).unwrap();

        let mut zome = Zome::default();
        zome.description = String::from("test");
        zome.config.error_handling = ErrorHandling::ThrowErrors;

        assert_eq!(fixture, zome);
    }
}
