use keys::Key;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentId {
    signing_key: Key,
    encryption_key: Key,
    dpki_root: Key,
}

impl AgentId {
    pub fn new(signing_key: &Key, encryption_key: &Key, dpki_root: &Key) -> AgentId {
        AgentId {
            signing_key: signing_key.to_owned(),
            encryption_key: encryption_key.to_owned(),
            dpki_root: dpki_root.to_owned(),
        }
    }
}
