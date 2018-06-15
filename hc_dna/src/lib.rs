//! hc_dna is a library for working with holochain dna files.
//!
//! It includes utilities for representing dna structures in memory,
//! as well as serializing and deserializing dna, mainly to json format.
//!
//! # Examples
//!
//! ```
//! use hc_dna::Dna;
//!
//! let name = String::from("My Holochain App");
//!
//! let mut dna = Dna::new();
//! dna.name = name.clone();
//!
//! let json = dna.to_json().unwrap();
//!
//! let dna2 = Dna::new_from_json(&json).unwrap();
//! assert_eq!(name, dna2.name);
//! ```

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate uuid;

use uuid::Uuid;

/// Enum for "zome" "config" "error_handling" property.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ZomeConfigErrorHandling {
    #[serde(rename = "throw-errors")]
    ThrowErrors,
}

/// Represents the "config" object on a "zome".
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ZomeConfig {
    /// How errors should be handled within this zome.
    pub error_handling: ZomeConfigErrorHandling,
}

impl Default for ZomeConfig {
    /// Provide defaults for the "zome" "config" object.
    fn default() -> Self {
        ZomeConfig {
            error_handling: ZomeConfigErrorHandling::ThrowErrors,
        }
    }
}

impl ZomeConfig {
    /// Allow sane defaults for `ZomeConfig::new()`.
    pub fn new() -> Self {
        Default::default()
    }
}

/// Enum for Zome EntryType "sharing" property.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ZomeEntryTypeSharing {
    #[serde(rename = "public")]
    Public,
    #[serde(rename = "private")]
    Private,
    #[serde(rename = "encrypted")]
    Encrypted,
}

/// Represents an individual object in the "zome" "entry_types" array.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ZomeEntryType {
    /// The name of this entry type.
    pub name: String,

    /// A description of this entry type.
    pub description: String,

    /// The sharing model of this entry type (public, private, encrypted).
    pub sharing: ZomeEntryTypeSharing,
}

impl Default for ZomeEntryType {
    /// Provide defaults for a "zome"s "entry_types" object.
    fn default() -> Self {
        ZomeEntryType {
            name: String::from(""),
            description: String::from(""),
            sharing: ZomeEntryTypeSharing::Public,
        }
    }
}

impl ZomeEntryType {
    /// Allow sane defaults for `ZomeEntryType::new()`.
    pub fn new() -> Self {
        Default::default()
    }
}

/// Represents an individual "zome".
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Zome {
    /// The name of this zome.
    pub name: String,

    /// A description of this zome.
    pub description: String,

    /// Configuration associated with this zome.
    pub config: ZomeConfig,

    /// An array of entry_types associated with this zome.
    pub entry_types: Vec<ZomeEntryType>,
}

impl Default for Zome {
    /// Provide defaults for an individual "zome".
    fn default() -> Self {
        Zome {
            name: String::from(""),
            description: String::from(""),
            config: ZomeConfig::new(),
            entry_types: Vec::new(),
        }
    }
}

impl Zome {
    /// Allow sane defaults for `Zome::new()`.
    pub fn new() -> Self {
        Default::default()
    }
}

/// Represents the top-level holochain dna object.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Dna {
    /// The top-level "name" of a holochain application.
    pub name: String,

    /// The top-level "description" of a holochain application.
    pub description: String,

    /// The semantic version of your holochain application.
    pub version: String,

    /// A unique identifier to distinguish your holochain application.
    pub uuid: String,

    /// Which version of the holochain dna spec does this represent?
    pub dna_spec_version: String,

    /// Any arbitrary application properties can be included in this object.
    pub properties: serde_json::Value,

    /// An array of zomes associated with your holochain application.
    pub zomes: Vec<Zome>,
}

impl Default for Dna {
    /// Provide defaults for a dna object.
    fn default() -> Self {
        Dna {
            name: String::from(""),
            description: String::from(""),
            version: String::from(""),
            uuid: Uuid::new_v4().to_string(),
            dna_spec_version: String::from("2.0"),
            properties: serde_json::Value::Object(serde_json::map::Map::new()),
            zomes: Vec::new(),
        }
    }
}

impl Dna {
    /// Allow sane defaults for `Dna::new()`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Create a new dna struct from a json string.
    pub fn new_from_json(dna: &str) -> serde_json::Result<Self> {
        serde_json::from_str(dna)
    }

    /// Generate a json string from an in-memory dna struct.
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }

    /// Generate a pretty-printed json string from an in-memory dna struct.
    pub fn to_json_pretty(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse_and_output_json() {
        let dna = Dna::new();

        let serialized = serde_json::to_string(&dna).unwrap();

        let deserialized: Dna = serde_json::from_str(&serialized).unwrap();

        assert_eq!(String::from("2.0"), deserialized.dna_spec_version);
    }

    #[test]
    fn can_parse_and_output_json_helpers() {
        let dna = Dna::new();

        let serialized = dna.to_json().unwrap();

        let deserialized = Dna::new_from_json(&serialized).unwrap();

        assert_eq!(String::from("2.0"), deserialized.dna_spec_version);
    }

    #[test]
    fn default_value_test() {
        let mut dna = Dna {
            uuid: String::from("00000000-0000-0000-0000-000000000000"),
            ..Default::default()
        };
        let mut zome = Zome::new();
        zome.entry_types.push(ZomeEntryType::new());
        dna.zomes.push(zome);

        let fixture = Dna::new_from_json(
            r#"{
            "name": "",
            "description": "",
            "version": "",
            "uuid": "00000000-0000-0000-0000-000000000000",
            "dna_spec_version": "2.0",
            "properties": {},
            "zomes": [
                {
                    "name": "",
                    "description": "",
                    "config": {
                        "error_handling": "throw-errors"
                    },
                    "entry_types": [
                        {
                            "name": "",
                            "description": "",
                            "sharing": "public"
                        }
                    ]
                }
            ]
        }"#,
        ).unwrap();

        assert_eq!(dna, fixture);
    }
}
