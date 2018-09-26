//! File holding all the structs for handling entry types defined by DNA.

/// Enum for Zome EntryType "sharing" property.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Hash)]
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
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Hash)]
pub struct LinksTo {
    /// The target_type of this links_to entry
    #[serde(default)]
    pub target_type: String,

    /// The tag of this links_to entry
    #[serde(default)]
    pub tag: String,
}

impl Default for LinksTo {
    /// Provide defaults for a "links_to" object.
    fn default() -> Self {
        LinksTo {
            target_type: String::new(),
            tag: String::new(),
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
}

impl Default for LinkedFrom {
    /// Provide defaults for a "links_to" object.
    fn default() -> Self {
        LinkedFrom {
            base_type: String::new(),
            tag: String::new(),
        }
    }
}

impl LinkedFrom {
    /// Allow sane defaults for `LinkedFrom::new()`.
    pub fn new() -> Self {
        Default::default()
    }
}

/// Represents an individual object in the "zome" "entry_types" array.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Hash)]
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

impl Default for EntryTypeDef {
    /// Provide defaults for a "zome"s "entry_types" object.
    fn default() -> Self {
        EntryTypeDef {
            description: String::new(),
            sharing: Sharing::Public,
            links_to: Vec::new(),
            linked_from: Vec::new(),
        }
    }
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
    fn build_and_compare() {
        let fixture: EntryTypeDef = serde_json::from_str(
            r#"{
                "description": "test",
                "sharing": "public",
                "links_to": [
                    {
                        "target_type": "test",
                        "tag": "test"
                    }
                ],
                "linked_from": [
                    {
                        "base_type": "HcSysAgentKeyHash",
                        "tag": "authored_posts"
                    }
                ]
            }"#,
        ).unwrap();

        let mut entry = EntryTypeDef::new();
        entry.description = String::from("test");
        entry.sharing = Sharing::Public;

        let mut link = LinksTo::new();
        link.target_type = String::from("test");
        link.tag = String::from("test");
        entry.links_to.push(link);

        let mut linked = LinkedFrom::new();
        linked.base_type = String::from("HcSysAgentKeyHash");
        linked.tag = String::from("authored_posts");
        entry.linked_from.push(linked);

        assert_eq!(fixture, entry);
    }
}
