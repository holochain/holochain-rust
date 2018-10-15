//! holochain_dna::zome is a set of structs for working with holochain dna.

pub mod capabilities;
pub mod entry_types;

use entry::{dna::wasm::DnaWasm, AppEntryType};
use std::collections::HashMap;

pub type ZomeName = String;
pub type ZomeDescription = String;

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

pub type AppEntryTypes = HashMap<AppEntryType, entry_types::EntryTypeDef>;

pub type Capabilities = HashMap<String, capabilities::Capability>;

/// Represents an individual "zome".
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Zome {
    /// A description of this zome.
    #[serde(default)]
    pub description: ZomeDescription,

    /// Configuration associated with this zome.
    /// Note, this should perhaps be a more free-form serde_json::Value,
    /// "throw-errors" may not make sense for wasm, or other ribosome types.
    #[serde(default)]
    pub config: Config,

    /// An array of entry_types associated with this zome.
    #[serde(default)]
    app_entry_types: AppEntryTypes,

    /// An array of capabilities associated with this zome.
    #[serde(default)]
    pub capabilities: Capabilities,

    /// Validation code for this entry_type.
    #[serde(default)]
    pub code: DnaWasm,
}

impl Eq for Zome {}

impl Default for Zome {
    /// Provide defaults for an individual "zome".
    fn default() -> Self {
        Zome {
            description: ZomeDescription::new(),
            config: Config::new(),
            app_entry_types: AppEntryTypes::new(),
            capabilities: Capabilities::new(),
            code: DnaWasm::new(),
        }
    }
}

impl Zome {
    /// Allow sane defaults for `Zome::new()`.
    pub fn new(
        description: &str,
        config: &Config,
        app_entry_types: &AppEntryTypes,
        capabilities: &Capabilities,
        code: &DnaWasm,
    ) -> Zome {
        Zome {
            description: description.into(),
            config: config.clone(),
            app_entry_types: app_entry_types.to_owned(),
            capabilities: capabilities.to_owned(),
            code: code.clone(),
        }
    }

    pub fn app_entry_types(&self) -> &HashMap<AppEntryType, entry_types::EntryTypeDef> {
        &self.app_entry_types
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use serde_json;
    use zome::Zome;

    pub fn test_zome() -> Zome {
        Zome::default()
    }

    #[test]
    fn build_and_compare() {
        let fixture: Zome = serde_json::from_str(
            r#"{
                "description": "test",
                "config": {
                    "error_handling": "throw-errors"
                },
                "entry_types": {},
                "capabilities": {}
            }"#,
        ).unwrap();

        let mut zome = Zome::default();
        zome.description = String::from("test");
        zome.config.error_handling = ErrorHandling::ThrowErrors;

        assert_eq!(fixture, zome);
    }
}
