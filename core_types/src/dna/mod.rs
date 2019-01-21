//! dna is a library for working with holochain dna files/entries.
//!
//! It includes utilities for representing dna structures in memory,
//! as well as serializing and deserializing dna, mainly to json format.
//!
//! # Examples
//!
//! ```
//! #![feature(try_from)]
//! extern crate holochain_core_types;
//! use holochain_core_types::dna::Dna;
//! use holochain_core_types::json::JsonString;
//! use std::convert::TryFrom;
//!
//! let name = String::from("My Holochain DNA");
//!
//! let mut dna = Dna::new();
//! dna.name = name.clone();
//!
//! let json = JsonString::from(dna.clone());
//!
//! let dna2 = Dna::try_from(json).expect("could not restore DNA from JSON");
//! assert_eq!(name, dna2.name);
//! ```

pub mod bridges;
pub mod capabilities;
pub mod dna;
pub mod entry_types;
pub mod wasm;
pub mod zome;

pub use dna::dna::Dna;

#[cfg(test)]
pub mod tests {
    use super::*;
    extern crate base64;
    use crate::{
        cas::content::Address,
        dna::{
            bridges::{Bridge, BridgePresence, BridgeReference},
            capabilities::{Capability, CapabilityType, FnDeclaration, FnParameter},
            entry_types::EntryTypeDef,
            zome::tests::test_zome,
        },
        entry::entry_type::{AppEntryType, EntryType},
        json::JsonString,
    };
    use std::convert::TryFrom;

    static UNIT_UUID: &'static str = "00000000-0000-0000-0000-000000000000";

    pub fn test_dna() -> Dna {
        Dna::new()
    }

    #[test]
    fn get_entry_type_def_test() {
        let mut dna = test_dna();
        let mut zome = test_zome();
        let entry_type = EntryType::App(AppEntryType::from("bar"));
        let entry_type_def = EntryTypeDef::new();

        zome.entry_types
            .insert(entry_type.into(), entry_type_def.clone());
        dna.zomes.insert("zome".to_string(), zome);

        assert_eq!(None, dna.get_entry_type_def("foo"));
        assert_eq!(Some(&entry_type_def), dna.get_entry_type_def("bar"));
    }

    #[test]
    fn can_parse_and_output_json() {
        let dna = test_dna();

        let serialized = serde_json::to_string(&dna).unwrap();

        let deserialized: Dna = serde_json::from_str(&serialized).unwrap();

        assert_eq!(String::from("2.0"), deserialized.dna_spec_version);
    }

    #[test]
    fn can_parse_and_output_json_helpers() {
        let dna = test_dna();

        let json_string = JsonString::from(dna);

        let deserialized = Dna::try_from(json_string).unwrap();

        assert_eq!(String::from("2.0"), deserialized.dna_spec_version);
    }

    #[test]
    fn parse_and_serialize_compare() {
        let fixture = String::from(
            r#"{
                "name": "test",
                "description": "test",
                "version": "test",
                "uuid": "00000000-0000-0000-0000-000000000000",
                "dna_spec_version": "2.0",
                "properties": {
                    "test": "test"
                },
                "zomes": {
                    "test": {
                        "description": "test",
                        "config": {},
                        "entry_types": {
                            "test": {
                                "description": "test",
                                "sharing": "public",
                                "links_to": [
                                    {
                                        "target_type": "test",
                                        "tag": "test"
                                    }
                                ],
                                "linked_from": []
                            }
                        },
                        "capabilities": {
                            "test": {
                                "type": "public",
                                "functions": [
                                    {
                                        "name": "test",
                                        "inputs": [],
                                        "outputs": []
                                    }
                                ]
                            }
                        },
                        "code": {
                            "code": "AAECAw=="
                        },
                        "bridges": []
                    }
                }
            }"#,
        )
        .replace(char::is_whitespace, "");

        let dna = Dna::try_from(JsonString::from(fixture.clone())).unwrap();

        println!("{}", dna.to_json_pretty().unwrap());

        let serialized = String::from(JsonString::from(dna)).replace(char::is_whitespace, "");

        assert_eq!(fixture, serialized);
    }

    #[test]
    fn default_value_test() {
        let mut dna = Dna {
            uuid: String::from(UNIT_UUID),
            ..Default::default()
        };
        let mut zome = zome::Zome::default();
        zome.entry_types
            .insert("".into(), entry_types::EntryTypeDef::new());
        dna.zomes.insert("".to_string(), zome);

        let expected = JsonString::from(dna.clone());
        println!("{:?}", expected);

        let fixture = Dna::try_from(JsonString::from(
            r#"{
                "name": "",
                "description": "",
                "version": "",
                "uuid": "00000000-0000-0000-0000-000000000000",
                "dna_spec_version": "2.0",
                "properties": {},
                "zomes": {
                    "": {
                        "description": "",
                        "config": {},
                        "entry_types": {
                            "": {
                                "description": "",
                                "sharing": "public",
                                "links_to": [],
                                "linked_from": []
                            }
                        },
                        "capabilities": {},
                        "code": {"code": ""}
                    }
                }
            }"#,
        ))
        .unwrap();

        assert_eq!(dna, fixture);
    }

    #[test]
    fn parse_with_defaults_dna() {
        let dna = Dna::try_from(JsonString::from(
            r#"{
            }"#,
        ))
        .unwrap();

        assert!(dna.uuid.len() > 0);
    }

    #[test]
    fn parse_with_defaults_entry_type() {
        let dna = Dna::try_from(JsonString::from(
            r#"{
                "zomes": {
                    "zome1": {
                        "entry_types": {
                            "type1": {}
                        }
                    }
                }
            }"#,
        ))
        .unwrap();

        assert_eq!(
            dna.zomes
                .get("zome1")
                .unwrap()
                .entry_types
                .get(&"type1".into())
                .unwrap()
                .sharing,
            entry_types::Sharing::Public
        );
    }

    #[test]
    fn parse_wasm() {
        let dna = Dna::try_from(JsonString::from(
            r#"{
                "zomes": {
                    "zome1": {
                        "entry_types": {
                            "type1": {}
                        },
                        "code": {
                            "code": "AAECAw=="
                        }
                    }
                }
            }"#,
        ))
        .unwrap();

        assert_eq!(vec![0, 1, 2, 3], dna.zomes.get("zome1").unwrap().code.code);
    }

    #[test]
    #[should_panic]
    fn parse_fail_if_bad_type_dna() {
        Dna::try_from(JsonString::from(
            r#"{
                "name": 42
            }"#,
        ))
        .unwrap();
    }

    #[test]
    #[should_panic]
    fn parse_fail_if_bad_type_zome() {
        Dna::try_from(JsonString::from(
            r#"{
                "zomes": {
                    "zome1": {
                        "description": 42
                    }
                }
            }"#,
        ))
        .unwrap();
    }

    #[test]
    #[should_panic]
    fn parse_fail_if_bad_type_entry_type() {
        Dna::try_from(JsonString::from(
            r#"{
                "zomes": {
                    "zome1": {
                        "entry_types": {
                            "test": {
                                "description": 42
                            }
                        }
                    }
                }
            }"#,
        ))
        .unwrap();
    }

    #[test]
    fn parse_accepts_arbitrary_dna_properties() {
        let dna = Dna::try_from(JsonString::from(
            r#"{
                "properties": {
                    "str": "hello",
                    "num": 3.14159,
                    "bool": true,
                    "null": null,
                    "arr": [1, 2],
                    "obj": {"a": 1, "b": 2}
                }
            }"#,
        ))
        .unwrap();

        let props = dna.properties.as_object().unwrap();

        assert_eq!("hello", props.get("str").unwrap().as_str().unwrap());
        assert_eq!(3.14159, props.get("num").unwrap().as_f64().unwrap());
        assert_eq!(true, props.get("bool").unwrap().as_bool().unwrap());
        assert!(props.get("null").unwrap().is_null());
        assert_eq!(
            1_i64,
            props.get("arr").unwrap().as_array().unwrap()[0]
                .as_i64()
                .unwrap()
        );
        assert_eq!(
            1_i64,
            props
                .get("obj")
                .unwrap()
                .as_object()
                .unwrap()
                .get("a")
                .unwrap()
                .as_i64()
                .unwrap()
        );
    }

    #[test]
    fn get_wasm_from_zome_name() {
        let dna = Dna::try_from(JsonString::from(
            r#"{
                "name": "test",
                "description": "test",
                "version": "test",
                "uuid": "00000000-0000-0000-0000-000000000000",
                "dna_spec_version": "2.0",
                "properties": {
                    "test": "test"
                },
                "zomes": {
                    "test zome": {
                        "name": "test zome",
                        "description": "test",
                        "config": {},
                        "entry_types": {},
                        "capabilities": {
                            "test capability": {
                                "type": "public",
                                "fn_declarations": [
                                    {
                                        "name": "test",
                                        "signature": {
                                            "inputs": [],
                                            "outputs": []
                                        }
                                    }
                                ]
                            }
                        },
                        "code": {
                            "code": "AAECAw=="
                        }
                    }
                }
            }"#,
        ))
        .unwrap();

        let wasm = dna.get_wasm_from_zome_name("test zome");
        assert_eq!("AAECAw==", base64::encode(&wasm.unwrap().code));

        let fail = dna.get_wasm_from_zome_name("non existant zome");
        assert_eq!(None, fail);
    }

    #[test]
    fn test_get_zome_name_for_entry_type() {
        let dna = Dna::try_from(JsonString::from(
            r#"{
                "name": "test",
                "description": "test",
                "version": "test",
                "uuid": "00000000-0000-0000-0000-000000000000",
                "dna_spec_version": "2.0",
                "properties": {
                    "test": "test"
                },
                "zomes": {
                    "test zome": {
                        "name": "test zome",
                        "description": "test",
                        "config": {},
                        "capabilities": {
                            "test capability": {
                                "type": "public",
                                "fn_declarations": []
                            }
                        },
                        "entry_types": {
                            "test type": {
                                "description": "",
                                "sharing": "public"
                            }
                        },
                        "code": {
                            "code": ""
                        }
                    }
                }
            }"#,
        ))
        .unwrap();

        assert_eq!(
            dna.get_zome_name_for_app_entry_type(&AppEntryType::from("test type"))
                .unwrap(),
            "test zome".to_string()
        );
        assert!(dna
            .get_zome_name_for_app_entry_type(&AppEntryType::from("non existant entry type"))
            .is_none());
    }

    #[test]
    fn test_get_required_bridges() {
        let dna = Dna::try_from(JsonString::from(
            r#"{
                "name": "test",
                "description": "test",
                "version": "test",
                "uuid": "00000000-0000-0000-0000-000000000000",
                "dna_spec_version": "2.0",
                "properties": {
                    "test": "test"
                },
                "zomes": {
                    "test zome": {
                        "name": "test zome",
                        "description": "test",
                        "config": {},
                        "capabilities": {
                            "test capability": {
                                "type": "public",
                                "fn_declarations": []
                            }
                        },
                        "entry_types": {
                            "test type": {
                                "description": "",
                                "sharing": "public"
                            }
                        },
                        "code": {
                            "code": ""
                        },
                        "bridges": [
                            {
                                "presence": "required",
                                "handle": "DPKI",
                                "reference": {
                                    "dna_address": "Qmabcdef1234567890"
                                }
                            },
                            {
                                "presence": "optional",
                                "handle": "Vault",
                                "reference": {
                                    "capabilities": {
                                        "persona_management": {
                                            "type": "public",
                                            "functions": [
                                                {
                                                    "name": "get_persona",
                                                    "inputs": [{"name": "domain", "type": "string"}],
                                                    "outputs": [{"name": "persona", "type": "json"}]
                                                }
                                            ]
                                        }
                                    }
                                }
                            },
                            {
                                "presence": "required",
                                "handle": "HCHC",
                                "reference": {
                                    "capabilities": {
                                        "happ_directory": {
                                            "type": "public",
                                            "functions": [
                                                {
                                                    "name": "get_happs",
                                                    "inputs": [],
                                                    "outputs": [{"name": "happs", "type": "json"}]
                                                }
                                            ]
                                        }
                                    }
                                }
                            }
                        ]
                    }
                }
            }"#,
        ))
        .unwrap();

        assert_eq!(
            dna.get_required_bridges(),
            vec![
                Bridge {
                    presence: BridgePresence::Required,
                    handle: String::from("DPKI"),
                    reference: BridgeReference::Address {
                        dna_address: Address::from("Qmabcdef1234567890"),
                    }
                },
                Bridge {
                    presence: BridgePresence::Required,
                    handle: String::from("HCHC"),
                    reference: BridgeReference::Capability {
                        capabilities: btreemap! {
                            String::from("happ_directory") => Capability{
                                cap_type: CapabilityType::Public,
                                functions: vec![
                                    FnDeclaration {
                                        name: String::from("get_happs"),
                                        inputs: vec![],
                                        outputs: vec![FnParameter{
                                            name: String::from("happs"),
                                            parameter_type: String::from("json"),
                                        }],
                                    }
                                ]
                            }
                        }
                    },
                },
            ]
        );
    }
}
