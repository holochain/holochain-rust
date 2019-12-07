//! holochain_core_types::dna::zome is a set of structs for working with holochain dna.

use crate::{
    dna::{
        bridges::{Bridge, BridgePresence},
        fn_declarations::{FnDeclaration, FnParameter, TraitFns},
        traits::ReservedTraitNames,
        wasm::DnaWasm,
    },
    entry::entry_type::EntryType,
};

use holochain_json_api::{error::JsonError, json::JsonString};

use dna::entry_types::{self, deserialize_entry_types, serialize_entry_types, EntryTypeDef};
use std::collections::BTreeMap;

/// Represents the "config" object on a "zome".
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Hash)]
pub struct Config {}

impl Default for Config {
    /// Provide defaults for the "zome" "config" object.
    fn default() -> Self {
        Config {}
    }
}

impl Config {
    /// Allow sane defaults for `Config::new()`.
    pub fn new() -> Self {
        Default::default()
    }
}

pub type ZomeEntryTypes = BTreeMap<EntryType, EntryTypeDef>;
pub type ZomeTraits = BTreeMap<String, TraitFns>;
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

    /// An array of traits defined in this zome.
    #[serde(default)]
    pub traits: ZomeTraits,

    /// An array of functions declared in this this zome.
    #[serde(default)]
    pub fn_declarations: ZomeFnDeclarations,

    /// Validation code for this entry_type.
    pub code: DnaWasm,

    /// A list of bridges to other DNAs that this DNA can use or depends on.
    #[serde(default)]
    pub bridges: Vec<Bridge>,
}

impl Eq for Zome {}

impl Zome {
    /// Provide defaults for an individual "zome".
    pub fn empty() -> Self {
        Zome {
            description: String::new(),
            config: Config::new(),
            entry_types: BTreeMap::new(),
            fn_declarations: Vec::new(),
            traits: BTreeMap::new(),
            code: DnaWasm::new_invalid(),
            bridges: Vec::new(),
        }
    }

    /// Allow sane defaults for `Zome::new()`.
    pub fn new(
        description: &str,
        config: &Config,
        entry_types: &BTreeMap<EntryType, entry_types::EntryTypeDef>,
        fn_declarations: &[FnDeclaration],
        traits: &BTreeMap<String, TraitFns>,
        code: &DnaWasm,
    ) -> Zome {
        Zome {
            description: description.into(),
            config: config.clone(),
            entry_types: entry_types.to_owned(),
            fn_declarations: fn_declarations.to_owned(),
            traits: traits.to_owned(),
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
    pub fn is_fn_public(&self, fn_name: &str) -> bool {
        let pub_trait = ReservedTraitNames::Public.as_str();
        self.traits.iter().any(|(trait_name, trait_fns)| {
            trait_name == pub_trait && trait_fns.functions.contains(&fn_name.to_owned())
        })
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
    use std::{collections::BTreeMap, convert::TryFrom, sync::Arc};

    pub fn test_zome() -> Zome {
        Zome::empty()
    }

    #[test]
    fn build_and_compare() {
        let fixture: Zome = serde_json::from_str(
            r#"{
                "description": "test",
                "config": {},
                "entry_types": {},
                "fn_delcarations": [],
                "traits": {},
                "code": {
                    "code": ""
                }
            }"#,
        )
        .unwrap();

        let mut zome = Zome::empty();
        zome.description = String::from("test");

        assert_eq!(fixture, zome);
    }

    #[test]
    fn zome_json_test() {
        let mut entry_types = BTreeMap::new();
        entry_types.insert(EntryType::from("foo"), EntryTypeDef::new());
        let mut zome = Zome::empty();
        zome.entry_types = entry_types;

        let expected = "{\"description\":\"\",\"config\":{},\"entry_types\":{\"foo\":{\"properties\":\"{}\",\"sharing\":\"public\",\"links_to\":[],\"linked_from\":[]}},\"traits\":{},\"fn_declarations\":[],\"code\":{\"code\":\"\"},\"bridges\":[]}";

        assert_eq!(
            JsonString::from_json(expected),
            JsonString::from(zome.clone()),
        );

        assert_eq!(
            zome,
            Zome::try_from(JsonString::from_json(expected)).unwrap(),
        );

        // Ensure that compressed large WASM gets round-tripped to [String, ...]
        zome.code.code = Arc::new(vec![
                0x40, 0x35, 0xe0, 0x39, 0xa6, 0x27, 0x8a, 0x62, 0x9f, 0xa8, 0x2b, 0x2b, 0x7e, 0xbb, 0xed, 0xdb,
                0xb0, 0x90, 0x01, 0xf4, 0x07, 0xa6, 0xe2, 0xd5, 0xd1, 0x30, 0x1f, 0xe8, 0x77, 0x1b, 0xf0, 0x08,
                0x14, 0x0b, 0x09, 0xf3, 0xfc, 0xec, 0x1f, 0x76, 0x3e, 0xcd, 0x94, 0xe0, 0xe3, 0xbc, 0xe6, 0xe4,
                0x36, 0xd1, 0x7e, 0x77, 0xd7, 0xc4, 0x89, 0x8d, 0x6f, 0x78, 0x9d, 0x0b, 0xba, 0xa4, 0x53, 0xc7,
                0x37, 0x76, 0xd1, 0xf8, 0x5e, 0x62, 0x29, 0xc2, 0x2d, 0x3f, 0x9a, 0x8d, 0xb0, 0x1f, 0xe4, 0x72,
                0xaf, 0x34, 0x1f, 0x60, 0xb1, 0x24, 0x88, 0x8c, 0x33, 0x0d, 0xd6, 0xd3, 0x5c, 0xb3, 0xa7, 0xcc,
                0xd3, 0x62, 0xb2, 0x67, 0x17, 0x5d, 0xe6, 0x7a, 0x99, 0x12, 0x2c, 0x06, 0xa9, 0xce, 0xbe, 0xa9,
                0xbb, 0xa3, 0x89, 0x26, 0x45, 0x1a, 0xcf, 0xef, 0x69, 0xd0, 0x22, 0x4a, 0x8f, 0xd6, 0x07, 0xed,
                0x24, 0xca, 0xfe, 0xbb, 0x7e, 0x31, 0x5f, 0xe0, 0x7d, 0x5e, 0x59, 0x40, 0x21, 0x5f, 0x29, 0xfb,
                0x6f, 0x8f, 0xe5, 0x3b, 0xaa, 0xe4, 0x45, 0x51, 0xa0, 0x37, 0x0a, 0x20, 0xc0, 0x93, 0xb5, 0x46,
                0x7f, 0xf3, 0x48, 0x20, 0x69, 0xcb, 0x46, 0x0a, 0x10, 0xa1, 0x49, 0xed, 0xc8, 0x60, 0xa4, 0xc9,
                0xe9, 0x91, 0x15, 0x2a, 0xf0, 0xdc, 0x38, 0x2a, 0x5e, 0xe1, 0xb8, 0xa9, 0xad, 0x31, 0x96, 0x98,
                0xc3, 0xe0, 0x74, 0x61, 0xe9, 0x15, 0xb8, 0x6c, 0x4d, 0x37, 0x9c, 0xcc, 0xb6, 0x15, 0x32, 0xb3,
                0x0c, 0x30, 0x8d, 0x15, 0xd6, 0x6e, 0x0b, 0x68, 0x80, 0x9c, 0x0d, 0xb5, 0xcd, 0xdc, 0x86, 0xcc,
                0x98, 0xed, 0x1d, 0xc1, 0x67, 0x31, 0x80, 0x7a, 0x17, 0x26, 0x7e, 0x57, 0x0e, 0xd4, 0x52, 0xc1,
                0xac, 0x29, 0x60, 0x36, 0x28, 0xe9, 0x5b, 0x3e, 0x87, 0x81, 0x46, 0xe5, 0x78, 0xa9, 0xdb, 0x42,
                0xcc, 0x2a, 0x70, 0x8a, 0x8e, 0x91, 0xe3, 0xbf, 0x48, 0x77, 0x66, 0x52, 0x46, 0x41, 0x8b, 0xc0,
                0x83, 0x9a, 0x43, 0xfb, 0xdf, 0xeb, 0x34, 0x10, 0x0c, 0x3f, 0x42, 0x0c, 0xa6, 0xec, 0x27, 0xff,
                0xb0, 0xb4, 0x5c, 0x17, 0x8d, 0x91, 0x2f, 0x07, 0x56, 0x78, 0xfa, 0x0a, 0xfb, 0x27, 0x83, 0x65,
                0xb5, 0xe9, 0x8c, 0xe3, 0x2d, 0x08, 0x34, 0xde, 0xcc, 0x51, 0x45, 0xb3, 0x47, 0x4b, 0x05, 0xbb,
                0xaf, 0xcb, 0xac, 0x62, 0x73, 0xd7, 0x0e, 0x78, 0x57, 0xc5, 0x8d, 0xbe, 0x91, 0x23, 0xe0, 0xce,
                0xeb, 0x6b, 0x54, 0x72, 0x77, 0x45, 0x10, 0xec, 0xfb, 0x08, 0x42, 0x77, 0x56, 0xa4, 0x5e, 0x50,
                0x6f, 0x77, 0xc5, 0xa8, 0x4f, 0x2f, 0xfd, 0x5a, 0xa5, 0x06, 0xdc, 0xdc, 0x17, 0xb3, 0x33, 0x1c,
                0x94, 0x86, 0x2e, 0xca, 0x28, 0x50, 0x49, 0xb9, 0x17, 0x9a, 0x01, 0x2d, 0xc8, 0x70, 0x0a, 0x79,
                0x24, 0xba, 0x7c, 0x65, 0x52, 0x85, 0x3a, 0x17, 0x91, 0x95, 0xb2, 0xb9, 0xbe, 0x90, 0xcb, 0x95,
                0x71, 0x14, 0xc6, 0xb2, 0x50, 0xe8, 0xe9, 0x33, 0x77, 0x4f, 0xea, 0x7e, 0x27, 0x3f, 0x16, 0x08,
                0x07, 0x3f, 0x44, 0xce, 0xbf, 0x64, 0x06, 0xf9, 0xa3, 0x6b, 0x32, 0x36, 0x5f, 0x84, 0x23, 0x13,
                0x07, 0x55, 0x79, 0x5f, 0xb6, 0x67, 0x24, 0x04, 0x53, 0x1f, 0x44, 0xeb, 0xa9, 0xd7, 0xad, 0xe1,
                0x1a, 0x0f, 0xb7, 0x55, 0x54, 0x03, 0xad, 0x76, 0xfd, 0xc6, 0xf6, 0x8a, 0x84, 0xb1, 0x3f, 0xd4,
                0x1b, 0x52, 0x3e, 0xdc, 0xd9, 0x65, 0x95, 0x07, 0xd6, 0xfd, 0xc4, 0x97, 0x82, 0x63, 0x64, 0xcd,
                0xd6, 0xc7, 0xd9, 0x25, 0xa8, 0x1f, 0x72, 0x04, 0xdc, 0x51, 0xd3, 0x9c, 0x89, 0x66, 0x63, 0xa9,
                0xd3, 0x60, 0xaa, 0x50, 0xfa, 0x2f, 0xaf, 0x52, 0xc7, 0xa1, 0x8f, 0x81, 0x28, 0x6a, 0x8f, 0xe5,
        ]);

        let expected_big_wasm = "{\"description\":\"\",\"config\":{},\"entry_types\":{\"foo\":{\"properties\":\"{}\",\"sharing\":\"public\",\"links_to\":[],\"linked_from\":[]}},\"traits\":{},\"fn_declarations\":[],\"code\":{\"code\":[\"H4sIAAAAAAAC/wEAAv/9QDXgOaYnimKfqCsrfrvt27CQAfQHpuLV0TAf6Hcb8AgUCwnz/Owfdj7NlODjvObkNtF+d9fEiY1veJ0LuqRTxzd20fheYinCLT+ajbAf5HKv\",\"NB9gsSSIjDMN1tNcs6fM02KyZxdd5nqZEiwGqc6+qbujiSZFGs/vadAiSo/WB+0kyv67fjFf4H1eWUAhXyn7b4/lO6rkRVGgNwogwJO1Rn/zSCBpy0YKEKFJ7chgpMnp\",\"kRUq8Nw4Kl7huKmtMZaYw+B0YekVuGxNN5zMthUyswwwjRXWbgtogJwNtc3chsyY7R3BZzGAehcmflcO1FLBrClgNijpWz6HgUbleKnbQswqcIqOkeO/SHdmUkZBi8CD\",\"mkP73+s0EAw/Qgym7Cf/sLRcF42RLwdWePoK+yeDZbXpjOMtCDTezFFFs0dLBbuvy6xic9cOeFfFjb6RI+DO62tUcndFEOz7CEJ3VqReUG93xahPL/1apQbc3BezMxyU\",\"hi7KKFBJuReaAS3IcAp5JLp8ZVKFOheRlbK5vpDLlXEUxrJQ6Okzd0/qfic/FggHP0TOv2QG+aNrMjZfhCMTB1V5X7ZnJARTH0Trqdet4RoPt1VUA612/cb2ioSxP9Qb\",\"Uj7c2WWVB9b9xJeCY2TN1sfZJagfcgTcUdOciWZjqdNgqlD6L69Sx6GPgShqj+WBTIVcAAIAAA==\"]},\"bridges\":[]}";

        assert_eq!(
            JsonString::from_json(expected_big_wasm),
            JsonString::from(zome.clone()),
        );
        assert_eq!(
            zome,
            Zome::try_from(JsonString::from_json(expected_big_wasm)).unwrap(),
        );
    }

    #[test]
    fn test_zome_add_fn_declaration() {
        let mut zome = Zome::empty();
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
        let mut zome = Zome::empty();
        zome.add_fn_declaration(String::from("test"), vec![], vec![]);
        let result = zome.get_function("foo func");
        assert!(result.is_none());
        let fun = zome.get_function("test").unwrap();
        assert_eq!(
            format!("{:?}", fun),
            "FnDeclaration { name: \"test\", inputs: [], outputs: [] }"
        );
    }
}
