use crate::config_files::Dht;
use semver::Version;
use serde_json::Value;

#[derive(Serialize, Deserialize, Clone)]
pub struct App {
    pub name: String,
    pub description: String,
    pub authors: Vec<Author>,
    pub version: Version,
    pub dht: Dht,
    pub properties: Value,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Author {
    identifier: String,
    public_key_source: String,
    signature: String,
}

impl Default for App {
    fn default() -> Self {
        App {
            name: "Holochain App Name".into(),
            description: "A Holochain app".into(),
            version: Version::new(0, 1, 0),
            authors: vec![Author {
                identifier: "Author Name <author@name.com>".into(),
                public_key_source: "".into(),
                signature: "".into(),
            }],
            dht: Dht {},
            properties: Default::default(),
        }
    }
}
