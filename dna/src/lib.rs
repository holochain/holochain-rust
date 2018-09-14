//! holochain_dna is a library for working with holochain dna files.
//!
//! It includes utilities for representing dna structures in memory,
//! as well as serializing and deserializing dna, mainly to json format.
//!
//! # Examples
//!
//! ```
//! use holochain_dna::Dna;
//!
//! let name = String::from("My Holochain App");
//!
//! let mut dna = Dna::new();
//! dna.name = name.clone();
//!
//! let json = dna.to_json();
//!
//! let dna2 = Dna::from_json_str(&json).unwrap();
//! assert_eq!(name, dna2.name);
//! ```

#[macro_use]
extern crate serde_derive;
extern crate serde;
#[macro_use]
extern crate serde_json;
extern crate base64;
extern crate uuid;

use serde_json::Value;
use std::hash::{Hash, Hasher};

pub mod wasm;
pub mod zome;

use std::collections::HashMap;
use uuid::Uuid;

/// serde helper, provides a default empty object
fn empty_object() -> Value {
    json!({})
}

/// serde helper, provides a default newly generated v4 uuid
fn new_uuid() -> String {
    Uuid::new_v4().to_string()
}

/// Represents the top-level holochain dna object.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Dna {
    /// The top-level "name" of a holochain application.
    #[serde(default)]
    pub name: String,

    /// The top-level "description" of a holochain application.
    #[serde(default)]
    pub description: String,

    /// The semantic version of your holochain application.
    #[serde(default)]
    pub version: String,

    /// A unique identifier to distinguish your holochain application.
    #[serde(default = "new_uuid")]
    pub uuid: String,

    /// Which version of the holochain dna spec does this represent?
    #[serde(default)]
    pub dna_spec_version: String,

    /// Any arbitrary application properties can be included in this object.
    #[serde(default = "empty_object")]
    pub properties: Value,

    /// An array of zomes associated with your holochain application.
    #[serde(default)]
    pub zomes: HashMap<String, zome::Zome>,
}

impl Default for Dna {
    /// Provide defaults for a dna object.
    fn default() -> Self {
        Dna {
            name: String::new(),
            description: String::new(),
            version: String::new(),
            uuid: new_uuid(),
            dna_spec_version: String::from("2.0"),
            properties: empty_object(),
            zomes: HashMap::new(),
        }
    }
}

impl Dna {
    /// Create a new in-memory dna structure with some default values.
    ///
    /// # Examples
    ///
    /// ```
    /// use holochain_dna::Dna;
    ///
    /// let dna = Dna::new();
    /// assert_eq!("", dna.name);
    ///
    /// ```
    pub fn new() -> Self {
        Default::default()
    }

    /// Create a new in-memory dna struct from a json string.
    ///
    /// # Examples
    ///
    /// ```
    /// use holochain_dna::Dna;
    ///
    /// let dna = Dna::from_json_str(r#"{
    ///     "name": "MyTestApp"
    /// }"#).expect("DNA should be valid");
    ///
    /// assert_eq!("MyTestApp", dna.name);
    /// ```
    pub fn from_json_str(dna: &str) -> serde_json::Result<Self> {
        serde_json::from_str(dna)
    }

    /// Generate a json string from an in-memory dna struct.
    ///
    /// # Examples
    ///
    /// ```
    /// use holochain_dna::Dna;
    ///
    /// let dna = Dna::new();
    /// println!("json: {}", dna.to_json());
    ///
    /// ```
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("DNA should serialize")
    }

    /// Generate a pretty-printed json string from an in-memory dna struct.
    ///
    /// # Examples
    ///
    /// ```
    /// use holochain_dna::Dna;
    ///
    /// let dna = Dna::new();
    /// println!("json: {}", dna.to_json_pretty().expect("DNA should serialize"));
    ///
    /// ```
    pub fn to_json_pretty(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }

    /// Return a Zome
    pub fn get_zome(&self, zome_name: &str) -> Option<&zome::Zome> {
        self.zomes.get(zome_name)
    }

    /// Return a Zome's WASM bytecode for a specified Capability
    pub fn get_capability<'a>(
        &'a self,
        zome: &'a zome::Zome,
        capability_name: &str,
    ) -> Option<&'a wasm::DnaWasm> {
        let capability = zome.capabilities.get(capability_name);
        Some(&capability?.code)
    }

    /// Find a Zome and return it's WASM bytecode for a specified Capability
    pub fn get_wasm_for_capability<T: Into<String>>(
        &self,
        zome_name: T,
        capability_name: T,
    ) -> Option<&wasm::DnaWasm> {
        let zome_name = zome_name.into();
        let capability_name = capability_name.into();
        let zome = self.get_zome(&zome_name)?;
        let capability = self.get_capability(&zome, &capability_name)?;
        Some(capability)
    }

    /// Return a Zome's WASM bytecode for the validation of an entry
    pub fn get_validation_bytecode_for_entry_type(
        &self,
        zome_name: &str,
        entry_type_name: &str,
    ) -> Option<&wasm::DnaWasm> {
        let zome = self.get_zome(zome_name)?;
        let entry_type = zome.entry_types.get(entry_type_name)?;
        Some(&entry_type.validation)
    }

    pub fn get_zome_name_for_entry_type(&self, entry_type: String) -> Option<String> {
        for (zome_name, zome) in &self.zomes {
            for (entry_type_name, _) in &zome.entry_types {
                if *entry_type_name == entry_type {
                    return Some(zome_name.clone());
                }
            }
        }

        None
    }
}

impl Hash for Dna {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let s = self.to_json();
        s.hash(state);
    }
}

impl PartialEq for Dna {
    fn eq(&self, other: &Dna) -> bool {
        // need to guarantee that PartialEq and Hash always agree
        self.to_json() == other.to_json()
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    extern crate base64;

    static UNIT_UUID: &'static str = "00000000-0000-0000-0000-000000000000";

    pub fn test_dna() -> Dna {
        Dna::new()
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

        let serialized = dna.to_json();

        let deserialized = Dna::from_json_str(&serialized).unwrap();

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
                        "config": {
                            "error_handling": "throw-errors"
                        },
                        "entry_types": {
                            "test": {
                                "description": "test",
                                "sharing": "public",
                                "validation": {
                                    "code": "AAECAw=="
                                },
                                "links_to": [
                                    {
                                        "target_type": "test",
                                        "tag": "test",
                                        "validation": {
                                            "code": "AAECAw=="
                                        }
                                    }
                                ],
                                "linked_from": []
                            }
                        },
                        "capabilities": {
                            "test": {
                                "capability": {
                                    "membrane": "public"
                                },
                                "functions": [
                                    {
                                        "name": "test",
                                        "inputs": [],
                                        "outputs": []
                                    }
                                ],
                                "code": {
                                    "code": "AAECAw=="
                                }
                            }
                        }
                    }
                }
            }"#,
        ).replace(char::is_whitespace, "");

        let dna = Dna::from_json_str(&fixture).unwrap();

        println!("{}", dna.to_json_pretty().unwrap());

        let serialized = dna.to_json().replace(char::is_whitespace, "");

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
            .insert("".to_string(), zome::entry_types::EntryType::new());
        dna.zomes.insert("".to_string(), zome);

        let fixture = Dna::from_json_str(
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
                        "config": {
                            "error_handling": "throw-errors"
                        },
                        "entry_types": {
                            "": {
                                "description": "",
                                "sharing": "public"
                            }
                        }
                    }
                }
            }"#,
        ).unwrap();

        assert_eq!(dna, fixture);
    }

    #[test]
    fn parse_with_defaults_dna() {
        let dna = Dna::from_json_str(
            r#"{
            }"#,
        ).unwrap();

        assert!(dna.uuid.len() > 0);
    }

    #[test]
    fn parse_with_defaults_zome() {
        let dna = Dna::from_json_str(
            r#"{
                "zomes": {
                    "zome1": {}
                }
            }"#,
        ).unwrap();

        assert_eq!(
            dna.zomes.get("zome1").unwrap().config.error_handling,
            zome::ErrorHandling::ThrowErrors
        )
    }

    #[test]
    fn parse_with_defaults_entry_type() {
        let dna = Dna::from_json_str(
            r#"{
                "zomes": {
                    "zome1": {
                        "entry_types": {
                            "type1": {}
                        }
                    }
                }
            }"#,
        ).unwrap();

        assert_eq!(
            dna.zomes
                .get("zome1")
                .unwrap()
                .entry_types
                .get("type1")
                .unwrap()
                .sharing,
            zome::entry_types::Sharing::Public
        );
    }

    #[test]
    fn parse_wasm() {
        let dna = Dna::from_json_str(
            r#"{
                "zomes": {
                    "zome1": {
                        "entry_types": {
                            "type1": {
                                "validation": {
                                    "code": "AAECAw=="
                                }
                            }
                        }
                    }
                }
            }"#,
        ).unwrap();

        assert_eq!(
            vec![0, 1, 2, 3],
            dna.zomes
                .get("zome1")
                .unwrap()
                .entry_types
                .get("type1")
                .unwrap()
                .validation
                .code
        );
    }

    #[test]
    #[should_panic]
    fn parse_fail_if_bad_type_dna() {
        Dna::from_json_str(
            r#"{
                "name": 42
            }"#,
        ).unwrap();
    }

    #[test]
    #[should_panic]
    fn parse_fail_if_bad_type_zome() {
        Dna::from_json_str(
            r#"{
                "zomes": {
                    "zome1": {
                        "description": 42
                    }
                }
            }"#,
        ).unwrap();
    }

    #[test]
    #[should_panic]
    fn parse_fail_if_bad_type_entry_type() {
        Dna::from_json_str(
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
        ).unwrap();
    }

    #[test]
    fn parse_accepts_arbitrary_dna_properties() {
        let dna = Dna::from_json_str(
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
        ).unwrap();

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
    fn get_wasm_for_capability() {
        let dna = Dna::from_json_str(
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
                                "capability": {
                                    "membrane": "public"
                                },
                                "fn_declarations": [
                                    {
                                        "name": "test",
                                        "signature": {
                                            "inputs": [],
                                            "outputs": []
                                        }
                                    }
                                ],
                                "code": {
                                    "code": "AAECAw=="
                                }
                            }
                        }
                    }
                }
            }"#,
        ).unwrap();

        let wasm = dna.get_wasm_for_capability("test zome", "test capability");
        assert_eq!("AAECAw==", base64::encode(&wasm.unwrap().code));

        let fail = dna.get_wasm_for_capability("non existant zome", "test capability");
        assert_eq!(None, fail);
    }

    #[test]
    fn get_wasm_for_entry_type() {
        let dna = Dna::from_json_str(
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
                                "capability": {
                                    "membrane": "public"
                                },
                                "fn_declarations": [],
                                "code": {
                                    "code": ""
                                }
                            }
                        },
                        "entry_types": {
                            "test type": {
                                "description": "",
                                "sharing": "public",
                                "validation": {
                                    "code": "AAECAw=="
                                }
                            }
                        }
                    }
                }
            }"#,
        ).unwrap();

        let wasm = dna.get_validation_bytecode_for_entry_type("test zome", "test type");
        assert_eq!("AAECAw==", base64::encode(&wasm.unwrap().code));

        let fail = dna.get_validation_bytecode_for_entry_type("tets zome", "non existing type");
        assert_eq!(None, fail);
    }

    #[test]
    fn teset_get_zome_name_for_entry_type() {
        let dna = Dna::from_json_str(
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
                                "capability": {
                                    "membrane": "public"
                                },
                                "fn_declarations": [],
                                "code": {
                                    "code": ""
                                }
                            }
                        },
                        "entry_types": {
                            "test type": {
                                "description": "",
                                "sharing": "public",
                                "validation": {
                                    "code": "AAECAw=="
                                }
                            }
                        }
                    }
                }
            }"#,
        ).unwrap();

        assert_eq!(dna.get_zome_name_for_entry_type("test type".to_string()), "test type".to_string());
        assert!(dna.get_zome_name_for_entry_type("non existant entry type".to_string()).is_none());
    }
}
