//! holochain_dna::zome::entry_types is a set of structs for working with holochain dna.

use wasm::DnaWasm;

/// Enum for Zome EntryType "sharing" property.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Sharing {
    #[serde(rename = "public")]
    Public,
    #[serde(rename = "private")]
    Private,
    #[serde(rename = "encrypted")]
    Encrypted,
}

impl Default for Sharing {
    /// Default zome entry_type sharing is "public"
    fn default() -> Self {
        Sharing::Public
    }
}

/// An individual object in a "links_to" array.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LinksTo {
    /// The target_type of this links_to entry
    #[serde(default)]
    pub target_type: String,

    /// The tag of this links_to entry
    #[serde(default)]
    pub tag: String,

    /// Validation code for this links_to.
    #[serde(default)]
    pub validation: DnaWasm,
}

impl Default for LinksTo {
    /// Provide defaults for a "links_to" object.
    fn default() -> Self {
        LinksTo {
            target_type: String::from(""),
            tag: String::from(""),
            validation: DnaWasm::new(),
        }
    }
}

impl LinksTo {
    /// Allow sane defaults for `LinksTo::new()`.
    pub fn new() -> Self {
        Default::default()
    }
}

/// Represents an individual object in the "zome" "entry_types" array.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct EntryType {
    /// The name of this entry type.
    #[serde(default)]
    pub name: String,

    /// A description of this entry type.
    #[serde(default)]
    pub description: String,

    /// The sharing model of this entry type (public, private, encrypted).
    #[serde(default)]
    pub sharing: Sharing,

    /// Validation code for this entry_type.
    #[serde(default)]
    pub validation: DnaWasm,

    /// An array of entry_types associated with this zome.
    #[serde(default)]
    pub links_to: Vec<LinksTo>,
}

impl Default for EntryType {
    /// Provide defaults for a "zome"s "entry_types" object.
    fn default() -> Self {
        EntryType {
            name: String::from(""),
            description: String::from(""),
            sharing: Sharing::Public,
            validation: DnaWasm::new(),
            links_to: Vec::new(),
        }
    }
}

impl EntryType {
    /// Allow sane defaults for `EntryType::new()`.
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
        let fixture: EntryType = serde_json::from_str(
            r#"{
                "name": "test",
                "description": "test",
                "validation": {
                    "code": "AAECAw=="
                },
                "sharing": "public",
                "links_to": [
                    {
                        "target_type": "test",
                        "tag": "test",
                        "validation": {
                            "code": "AAECAw=="
                        }
                    }
                ]
            }"#,
        ).unwrap();

        let mut entry = EntryType::new();
        entry.name = String::from("test");
        entry.description = String::from("test");
        entry.validation.code = vec![0, 1, 2, 3];
        entry.sharing = Sharing::Public;

        let mut link = LinksTo::new();
        link.target_type = String::from("test");
        link.tag = String::from("test");
        link.validation.code = vec![0, 1, 2, 3];

        entry.links_to.push(link);

        assert_eq!(fixture, entry);
    }
}
