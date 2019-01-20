//! holochain_core_types::dna::zome is a set of structs for working with holochain dna.

use crate::{
    dna::{
        bridges::{Bridge, BridgePresence},
        capabilities::{Capability, CapabilityType},
        fn_declarations::{FnDeclaration, FnParameter},
        wasm::DnaWasm,
    },
    entry::entry_type::EntryType,
    error::HolochainError,
    json::JsonString,
};
use dna::entry_types::{self, deserialize_entry_types, serialize_entry_types, EntryTypeDef};
use std::collections::BTreeMap;

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

pub type ZomeEntryTypes = BTreeMap<EntryType, EntryTypeDef>;
pub type ZomeCapabilities = BTreeMap<String, Capability>;
pub type ZomeFnDeclarations = Vec<FnDeclaration>;

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
    pub entry_types: ZomeEntryTypes,

    /// An array of capabilities associated with this zome.
    #[serde(default)]
    pub capabilities: ZomeCapabilities,

    /// An array of functions declared in this this zome.
    #[serde(default)]
    pub fn_declarations: ZomeFnDeclarations,

    /// Validation code for this entry_type.
    #[serde(default)]
    pub code: DnaWasm,

    /// A list of bridges to other DNAs that this DNA can use or depends on.
    #[serde(default)]
    pub bridges: Vec<Bridge>,
}

impl Eq for Zome {}

impl Default for Zome {
    /// Provide defaults for an individual "zome".
    fn default() -> Self {
        Zome {
            description: String::new(),
            config: Config::new(),
            entry_types: BTreeMap::new(),
            fn_declarations: Vec::new(),
            capabilities: BTreeMap::new(),
            code: DnaWasm::new(),
            bridges: Vec::new(),
        }
    }
}

impl Zome {
    /// Allow sane defaults for `Zome::new()`.
    pub fn new(
        description: &str,
        config: &Config,
        entry_types: &BTreeMap<EntryType, entry_types::EntryTypeDef>,
        fn_declarations: &Vec<FnDeclaration>,
        capabilities: &BTreeMap<String, Capability>,
        code: &DnaWasm,
    ) -> Zome {
        Zome {
            description: description.into(),
            config: config.clone(),
            entry_types: entry_types.to_owned(),
            fn_declarations: fn_declarations.to_owned(),
            capabilities: capabilities.to_owned(),
            code: code.clone(),
            bridges: Vec::new(),
        }
    }

    pub fn get_required_bridges(&self) -> Vec<Bridge> {
        self.bridges
            .iter()
            .filter(|bridge| bridge.presence == BridgePresence::Required)
            .cloned()
            .collect()
    }

    /// Add a function declaration to a Zome
    pub fn add_fn_declaration(
        &mut self,
        name: String,
        inputs: Vec<FnParameter>,
        outputs: Vec<FnParameter>,
    ) {
        self.fn_declarations.push(FnDeclaration {
            name,
            inputs,
            outputs,
        });
    }

    /// Return a Function declaration from a Zome
    pub fn get_function(&self, fn_name: &str) -> Option<&FnDeclaration> {
        self.fn_declarations
            .iter()
            .find(|ref fn_decl| fn_decl.name == fn_name)
    }

    // Helper function for finding out if a given function call is public
    pub fn is_fn_public(&self, fn_name: &String) -> bool {
        self.capabilities
            .iter()
            .find(|(_, cap)| {
                cap.cap_type == CapabilityType::Public && cap.functions.contains(fn_name)
            })
            .is_some()
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::dna::{
        fn_declarations::FnParameter,
        zome::{entry_types::EntryTypeDef, Zome},
    };
    use serde_json;
    use std::{collections::BTreeMap, convert::TryFrom};

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
                "fn_delcarations": [],
                "capabilities": {}
            }"#,
        )
        .unwrap();

        let mut zome = Zome::default();
        zome.description = String::from("test");
        zome.config.error_handling = ErrorHandling::ThrowErrors;

        assert_eq!(fixture, zome);
    }

    #[test]
    fn zome_json_test() {
        let mut entry_types = BTreeMap::new();
        entry_types.insert(EntryType::from("foo"), EntryTypeDef::new());
        let zome = Zome {
            entry_types,
            ..Default::default()
        };

        let expected = "{\"description\":\"\",\"config\":{\"error_handling\":\"throw-errors\"},\"entry_types\":{\"foo\":{\"description\":\"\",\"sharing\":\"public\",\"links_to\":[],\"linked_from\":[]}},\"capabilities\":{},\"fn_declarations\":[],\"code\":{\"code\":\"\"},\"bridges\":[]}";

        assert_eq!(
            JsonString::from(expected.clone()),
            JsonString::from(zome.clone()),
        );

        assert_eq!(zome, Zome::try_from(JsonString::from(expected)).unwrap(),);
    }

    #[test]
    fn test_zome_add_fn_declaration() {
        let mut zome = Zome::default();
        assert_eq!(zome.fn_declarations.len(), 0);
        zome.add_fn_declaration(
            String::from("hello"),
            vec![],
            vec![FnParameter {
                name: String::from("greeting"),
                parameter_type: String::from("String"),
            }],
        );
        assert_eq!(zome.fn_declarations.len(), 1);

        let expected = "[FnDeclaration { name: \"hello\", inputs: [], outputs: [FnParameter { parameter_type: \"String\", name: \"greeting\" }] }]";
        assert_eq!(expected, format!("{:?}", zome.fn_declarations),);
    }

    #[test]
    fn test_zome_get_function() {
        let mut zome = Zome::default();
        zome.add_fn_declaration(String::from("test"), vec![], vec![]);
        let result = zome.get_function("foo func");
        assert!(result.is_none());
        let fun = zome.get_function("test").unwrap();
        assert_eq!(
            format!("{:?}", fun),
            "FnDeclaration { name: \"test\", inputs: [], outputs: [] }"
        );
    }

    #[test]
    fn test_is_fn_public() {
        let mut dna = test_utils::create_test_dna_with_wat("test_zome", "test_cap", None);
        let fn_name = String::from("public_test_fn");
        assert!(dna.get_zome("test_zome").unwrap().is_fn_public(&fn_name));

        let private_fn_name = String::from("non_pub_fn");
        dna.zomes.get_mut("test_zome").unwrap().add_fn_declaration(
            private_fn_name.clone(),
            vec![],
            vec![],
        );

        assert!(!dna
            .get_zome("test_zome")
            .unwrap()
            .is_fn_public(&private_fn_name));
    }

}
