/*!
hc_dna is a library for working with holochain dna files.

It includes utilities for representing dna structures in memory,
as well as serializing and deserializing dna, mainly to json format.

# Examples

```
use hc_dna::Dna;

let name = String::from("My Holochain App");

let mut dna = Dna::new();
dna.name = name.clone();

let json = dna.to_json().unwrap();

let dna2 = Dna::new_from_json(&json).unwrap();
assert_eq!(name, dna2.name);
```
*/

extern crate base64;
#[macro_use]
extern crate serde_derive;
extern crate serde;
#[macro_use]
extern crate serde_json;
extern crate uuid;

use serde::de::{Deserializer, Visitor};
use serde::ser::Serializer;
use uuid::Uuid;

fn _vec_u8_to_b64_str<S>(data: &Vec<u8>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let b64 = base64::encode(data);
    s.serialize_str(&b64)
}

fn _b64_str_to_vec_u8<'de, D>(d: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    struct Z;

    impl<'de> Visitor<'de> for Z {
        type Value = Vec<u8>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("string")
        }

        fn visit_str<E>(self, value: &str) -> Result<Vec<u8>, E>
        where
            E: serde::de::Error,
        {
            match base64::decode(value) {
                Ok(v) => Ok(v),
                Err(_) => Err(serde::de::Error::custom(String::from("nope"))),
            }
        }
    }

    d.deserialize_any(Z)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DnaWasm {
    #[serde(serialize_with = "_vec_u8_to_b64_str", deserialize_with = "_b64_str_to_vec_u8")]
    code: Vec<u8>,
    // using a struct gives us the flexibility to extend it later
    // should we need additional properties, like:
    // `filename: String,`
}

impl Default for DnaWasm {
    /// Provide defaults for wasm entries in dna structs.
    fn default() -> Self {
        DnaWasm {
            code: vec![0, 1, 2, 3],
        }
    }
}

impl DnaWasm {
    /// Allow sane defaults for `DnaWasm::new()`.
    pub fn new() -> Self {
        Default::default()
    }
}

/// Enum for "zome" "config" "error_handling" property.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ZomeConfigErrorHandling {
    #[serde(rename = "throw-errors")]
    ThrowErrors,
}

impl Default for ZomeConfigErrorHandling {
    /// Default zome config error_handling is "throw-errors"
    fn default() -> Self {
        ZomeConfigErrorHandling::ThrowErrors
    }
}

/// Represents the "config" object on a "zome".
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ZomeConfig {
    /// How errors should be handled within this zome.
    #[serde(default)]
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

impl Default for ZomeEntryTypeSharing {
    /// Default zome entry_type sharing is "public"
    fn default() -> Self {
        ZomeEntryTypeSharing::Public
    }
}

/// Represents an individual object in the "zome" "entry_types" array.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ZomeEntryType {
    /// The name of this entry type.
    #[serde(default)]
    pub name: String,

    /// A description of this entry type.
    #[serde(default)]
    pub description: String,

    /// The sharing model of this entry type (public, private, encrypted).
    #[serde(default)]
    pub sharing: ZomeEntryTypeSharing,

    #[serde(default)]
    pub validation: DnaWasm,
}

impl Default for ZomeEntryType {
    /// Provide defaults for a "zome"s "entry_types" object.
    fn default() -> Self {
        ZomeEntryType {
            name: String::from(""),
            description: String::from(""),
            sharing: ZomeEntryTypeSharing::Public,
            validation: DnaWasm::new(),
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
    #[serde(default)]
    pub name: String,

    /// A description of this zome.
    #[serde(default)]
    pub description: String,

    /// Configuration associated with this zome.
    #[serde(default)]
    pub config: ZomeConfig,

    /// An array of entry_types associated with this zome.
    #[serde(default)]
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

fn _def_empty_object() -> serde_json::Value {
    json!({})
}

fn _def_new_uuid() -> String {
    Uuid::new_v4().to_string()
}

/// Represents the top-level holochain dna object.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
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
    #[serde(default = "_def_new_uuid")]
    pub uuid: String,

    /// Which version of the holochain dna spec does this represent?
    #[serde(default)]
    pub dna_spec_version: String,

    /// Any arbitrary application properties can be included in this object.
    #[serde(default = "_def_empty_object")]
    pub properties: serde_json::Value,

    /// An array of zomes associated with your holochain application.
    #[serde(default)]
    pub zomes: Vec<Zome>,
}

impl Default for Dna {
    /// Provide defaults for a dna object.
    fn default() -> Self {
        Dna {
            name: String::from(""),
            description: String::from(""),
            version: String::from(""),
            uuid: _def_new_uuid(),
            dna_spec_version: String::from("2.0"),
            properties: _def_empty_object(),
            zomes: Vec::new(),
        }
    }
}

impl Dna {
    /**
    Create a new in-memory dna structure with some default values.

    # Examples

    ```
    use hc_dna::Dna;

    let dna = Dna::new();
    assert_eq!("", dna.name);

    ```
    */
    pub fn new() -> Self {
        Default::default()
    }

    /**
    Create a new in-memory dna struct from a json string.

    # Examples

    ```
    use hc_dna::Dna;

    let dna = Dna::new_from_json(r#"{
        "name": "MyTestApp"
    }"#).unwrap();

    assert_eq!("MyTestApp", dna.name);
    ```
    */
    pub fn new_from_json(dna: &str) -> serde_json::Result<Self> {
        serde_json::from_str(dna)
    }

    /**
    Generate a json string from an in-memory dna struct.

    # Examples

    ```
    use hc_dna::Dna;

    let dna = Dna::new();
    println!("json: {}", dna.to_json().unwrap());

    ```
    */
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }

    /**
    Generate a pretty-printed json string from an in-memory dna struct.

    # Examples

    ```
    use hc_dna::Dna;

    let dna = Dna::new();
    println!("json: {}", dna.to_json_pretty().unwrap());

    ```
    */
    pub fn to_json_pretty(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static UNIT_UUID: &'static str = "00000000-0000-0000-0000-000000000000";

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
            uuid: String::from(UNIT_UUID),
            ..Default::default()
        };
        let mut zome = Zome::new();
        zome.entry_types.push(ZomeEntryType::new());
        dna.zomes.push(zome);

        println!("oeu {}", dna.to_json_pretty().unwrap());

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

    #[test]
    fn parse_with_defaults_dna() {
        let dna = Dna::new_from_json(
            r#"{
            }"#,
        ).unwrap();

        assert!(dna.uuid.len() > 0);
    }

    #[test]
    fn parse_with_defaults_zome() {
        let dna = Dna::new_from_json(
            r#"{
                "zomes": [
                    {}
                ]
            }"#,
        ).unwrap();

        assert_eq!(
            dna.zomes[0].config.error_handling,
            ZomeConfigErrorHandling::ThrowErrors
        )
    }

    #[test]
    fn parse_with_defaults_entry_type() {
        let dna = Dna::new_from_json(
            r#"{
                "zomes": [
                    {
                        "entry_types": [
                            {}
                        ]
                    }
                ]
            }"#,
        ).unwrap();

        assert_eq!(
            dna.zomes[0].entry_types[0].sharing,
            ZomeEntryTypeSharing::Public
        );
    }

    #[test]
    fn parse_wasm() {
        let dna = Dna::new_from_json(
            r#"{
                "zomes": [
                    {
                        "entry_types": [
                            {
                                "validation": {
                                    "code": "AAECAw=="
                                }
                            }
                        ]
                    }
                ]
            }"#,
        ).unwrap();

        assert_eq!(
            vec![0, 1, 2, 3],
            dna.zomes[0].entry_types[0].validation.code
        );
    }

    #[test]
    #[should_panic]
    fn parse_fail_if_bad_type_dna() {
        Dna::new_from_json(
            r#"{
                "name": 42
            }"#,
        ).unwrap();
    }

    #[test]
    #[should_panic]
    fn parse_fail_if_bad_type_zome() {
        Dna::new_from_json(
            r#"{
                "zomes": [
                    {
                        "name": 42
                    }
                ]
            }"#,
        ).unwrap();
    }

    #[test]
    #[should_panic]
    fn parse_fail_if_bad_type_entry_type() {
        Dna::new_from_json(
            r#"{
                "zomes": [
                    {
                        "entry_types": [
                            {
                                "name": 42
                            }
                        ]
                    }
                ]
            }"#,
        ).unwrap();
    }

    #[test]
    fn parse_accepts_arbitrary_dna_properties() {
        let dna = Dna::new_from_json(
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
}
