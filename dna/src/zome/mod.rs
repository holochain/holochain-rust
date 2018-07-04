//! holochain_dna::zome is a set of structs for working with holochain dna.

pub mod capabilities;
pub mod entry_types;

/// Enum for "zome" "config" "error_handling" property.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
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
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
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
    /// The name of this zome.
    #[serde(default)]
    pub name: String,

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
    pub entry_types: Vec<entry_types::EntryType>,

    /// An array of capabilities associated with this zome.
    #[serde(default)]
    pub capabilities: Vec<capabilities::Capability>,
}

impl Default for Zome {
    /// Provide defaults for an individual "zome".
    fn default() -> Self {
        Zome {
            name: String::from(""),
            description: String::from(""),
            config: Config::new(),
            entry_types: Vec::new(),
            capabilities: Vec::new(),
        }
    }
}

impl Zome {
    /// Allow sane defaults for `Zome::new()`.
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
        let fixture: Zome = serde_json::from_str(
            r#"{
                "name": "test",
                "description": "test",
                "config": {
                    "error_handling": "throw-errors"
                },
                "entry_types": [],
                "capabilities": []
            }"#,
        ).unwrap();

        let mut zome = Zome::new();
        zome.name = String::from("test");
        zome.description = String::from("test");
        zome.config.error_handling = ErrorHandling::ThrowErrors;

        assert_eq!(fixture, zome);
    }
}
