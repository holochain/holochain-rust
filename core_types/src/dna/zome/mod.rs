//! holochain_core_types::dna::zome is a set of structs for working with holochain dna.

pub mod capabilities;
pub mod entry_types;

use dna::{wasm::DnaWasm, zome::entry_types::EntryTypeDef};
use entry::entry_type::EntryType;
use error::HolochainError;
use json::JsonString;
use serde::{ser::SerializeMap, Deserialize, Deserializer, Serializer};
use std::collections::HashMap;

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

fn serialize_entry_types<S>(
    entry_types: &HashMap<EntryType, entry_types::EntryTypeDef>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut map = serializer.serialize_map(Some(entry_types.len()))?;
    for (k, v) in entry_types {
        map.serialize_entry(&String::from(k.to_owned()), &v)?;
    }
    map.end()
}

fn deserialize_entry_types<'de, D>(
    deserializer: D,
) -> Result<(HashMap<EntryType, entry_types::EntryTypeDef>), D::Error>
where
    D: Deserializer<'de>,
{
    // type SerializedEntryTypes = ;

    let serialized_entry_types = HashMap::<String, EntryTypeDef>::deserialize(deserializer)?;

    let mut map = HashMap::new();
    for (k, v) in serialized_entry_types {
        map.insert(EntryType::from(k), v);
    }
    Ok(map)
}

/// Represents an individual "zome".
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, DefaultJson)]
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
    #[serde(serialize_with = "serialize_entry_types")]
    #[serde(deserialize_with = "deserialize_entry_types")]
    pub entry_types: HashMap<EntryType, entry_types::EntryTypeDef>,

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
        entry_types: &HashMap<EntryType, entry_types::EntryTypeDef>,
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
    use dna::zome::{entry_types::EntryTypeDef, ErrorHandling, Zome};
    use entry::entry_type::EntryType;
    use json::JsonString;
    use serde_json;
    use std::{collections::HashMap, convert::TryFrom};

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

    #[test]
    fn zome_json_test() {
        let mut entry_types = HashMap::new();
        entry_types.insert(EntryType::from("foo"), EntryTypeDef::new());
        let zome = Zome {
            entry_types,
            ..Default::default()
        };

        let expected = "{\"description\":\"\",\"config\":{\"error_handling\":\"throw-errors\"},\"entry_types\":{\"foo\":{\"description\":\"\",\"sharing\":\"public\",\"links_to\":[],\"linked_from\":[]}},\"capabilities\":{},\"code\":{\"code\":\"\"}}";

        assert_eq!(
            JsonString::from(expected.clone()),
            JsonString::from(zome.clone()),
        );

        assert_eq!(zome, Zome::try_from(JsonString::from(expected)).unwrap(),);
    }
}
