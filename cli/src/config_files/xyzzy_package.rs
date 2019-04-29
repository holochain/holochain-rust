use crate::config_files::Dht;
use semver::Version;
use serde_json::Value;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
pub struct XyzzyPackage {
    pub name: String,
    pub description: String,
    pub authors: Vec<Author>,
    pub version: Version,
    pub dht: Dht,
    pub properties: Value,
    pub zomes: Vec<PathBuf>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Author {
    identifier: String,
    public_key_source: String,
    signature: String,
}

impl Default for XyzzyPackage {
    fn default() -> Self {
        XyzzyPackage {
            name: "Holochain XYZZY Name".into(),
            description: "Just another Holochain XYZZY".into(),
            version: Version::new(0, 1, 0),
            authors: vec![Author {
                identifier: "Author Name <author@name.com>".into(),
                public_key_source: "".into(),
                signature: "".into(),
            }],
            dht: Dht {},
            properties: Default::default(),
            zomes: Vec::new(),
        }
    }
}
