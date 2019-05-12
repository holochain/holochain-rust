//! File holding all the structs for handling entry types defined by DNA.

use dna::zome::ZomeEntryTypes;
use entry::entry_type::EntryType;
use error::HolochainError;
use json::JsonString;
use serde::{ser::SerializeMap, Deserialize, Deserializer, Serializer};
use std::collections::BTreeMap;

/// Enum for Zome EntryType "sharing" property.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Sharing {
    Public,
    Private,
    Encrypted,
}

impl Sharing {
    #[rustfmt::skip]
    pub fn can_publish(self) -> bool {
       match self {
           Sharing::Public    => true,
           Sharing::Private   => false,
           Sharing::Encrypted => true,
       }
    }
}

impl Default for Sharing {
    /// Default zome entry_type sharing is "public"
    fn default() -> Self {
        Sharing::Public
    }
}

/// An individual object in a "links_to" array.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Hash)]
pub struct LinksTo {
    /// The target_type of this links_to entry
    #[serde(default)]
    pub target_type: String,

    /// The tag of this links_to entry
    #[serde(default)]
    pub tag: String,

    #[serde(default)]
    pub r#type: String,
}

impl Default for LinksTo {
    /// Provide defaults for a "links_to" object.
    fn default() -> Self {
        LinksTo {
            target_type: String::new(),
            tag: String::new(),
            r#type: String::new(),
        }
    }
}

impl LinksTo {
    /// Allow sane defaults for `LinksTo::new()`.
    pub fn new() -> Self {
        Default::default()
    }
}

/// An a definition of a link from another type (including anchors and system hashes)
/// to the entry type it is part of.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Hash)]
pub struct LinkedFrom {
    /// The target_type of this links_to entry
    #[serde(default)]
    pub base_type: String,

    /// The tag of this links_to entry
    #[serde(default)]
    pub tag: String,

    #[serde(default)]
    pub r#type: String,
}

impl Default for LinkedFrom {
    /// Provide defaults for a "links_to" object.
    fn default() -> Self {
        LinkedFrom {
            base_type: String::new(),
            tag: String::new(),
            r#type: String::new(),
        }
    }
}

impl LinkedFrom {
    /// Allow sane defaults for `LinkedFrom::new()`.
    pub fn new() -> Self {
        Default::default()
    }
}

pub fn serialize_entry_types<S>(
    entry_types: &ZomeEntryTypes,
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

pub fn deserialize_entry_types<'de, D>(deserializer: D) -> Result<ZomeEntryTypes, D::Error>
where
    D: Deserializer<'de>,
{
    let serialized_entry_types: BTreeMap<String, EntryTypeDef> =
        BTreeMap::deserialize(deserializer)?;

    Ok(serialized_entry_types
        .into_iter()
        .map(|(k, v)| (EntryType::from(k), v))
        .collect())
}

/// Represents an individual object in the "zome" "entry_types" array.
#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq, Hash, DefaultJson)]
pub struct EntryTypeDef {
    /// A description of this entry type.
    #[serde(default)]
    pub description: String,

    /// The sharing model of this entry type (public, private, encrypted).
    #[serde(default)]
    pub sharing: Sharing,

    /// An array of link definitions associated with this entry type
    #[serde(default)]
    pub links_to: Vec<LinksTo>,

    /// An array of link definitions for links pointing to entries of this type
    #[serde(default)]
    pub linked_from: Vec<LinkedFrom>,
}

impl EntryTypeDef {
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
    fn can_publish() {
        assert!(Sharing::Public.can_publish());
        assert!(!Sharing::Private.can_publish());
    }

    #[test]
    fn build_and_compare() {
        let fixture: EntryTypeDef = serde_json::from_str(
            r#"{
                "description": "test",
                "sharing": "public",
                "links_to": [
                    {
                        "target_type": "test",
                        "tag": "test",
                        "type": "test"
                    }
                ],
                "linked_from": [
                    {
                        "base_type": "HcSysAgentKeyHash",
                        "tag": "authored_posts",
                        "type": "authored_posts_type"
                    }
                ]
            }"#,
        )
        .unwrap();

        let mut entry = EntryTypeDef::new();
        entry.description = String::from("test");
        entry.sharing = Sharing::Public;

        let mut link = LinksTo::new();
        link.target_type = String::from("test");
        link.tag = String::from("test");
        link.r#type = String::from("test");
        entry.links_to.push(link);

        let mut linked = LinkedFrom::new();
        linked.base_type = String::from("HcSysAgentKeyHash");
        linked.tag = String::from("authored_posts");
        linked.r#type = String::from("authored_posts_type");
        entry.linked_from.push(linked);

        assert_eq!(fixture, entry);
    }
}
