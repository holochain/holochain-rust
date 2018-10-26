#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
/// represents a single Key
/// e.g. private + public keys would be two Key structs
pub struct Key {}

impl Key {
    /// returns a new agent Key
    pub fn new() -> Key {
        Key {}
    }
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
/// represents a set of Keys for an agent
/// includes both public and private keys
/// also includes the node id of the agent with these keys
pub struct Keys {
    public_key: Key,
    private_key: Key,
    node_id: String,
}

impl Keys {
    /// returns a new set of agent Keys
    pub fn new<S: Into<String>>(public_key: &Key, private_key: &Key, node_id: S) -> Keys {
        Keys {
            public_key: public_key.clone(),
            private_key: private_key.clone(),
            node_id: node_id.into(),
        }
    }

    /// getter for the public key
    pub fn public_key(&self) -> Key {
        self.public_key.clone()
    }

    /// getter for the private key
    pub fn private_key(&self) -> Key {
        self.private_key.clone()
    }

    /// getter for the node id
    pub fn node_id(&self) -> String {
        self.node_id.clone()
    }
}

/// generates a new key suitable for testing
pub fn test_key() -> Key {
    Key::new()
}

/// dummy public key
pub fn test_public_key() -> Key {
    test_key()
}

/// dummy private key
pub fn test_private_key() -> Key {
    test_key()
}

/// generates a new node id suitable for testing
pub fn test_node_id() -> String {
    "test node id".into()
}

/// generates new id/pub/priv keys suitable for testing
pub fn test_keys() -> Keys {
    Keys::new(&test_key(), &test_key(), test_node_id())
}

#[cfg(test)]
pub mod tests {

    use super::*;

    #[test]
    /// smoke test new key
    fn key_new() {
        test_key();
    }

    #[test]
    /// smoke test new keys
    fn keys_new() {
        test_keys();
    }

    #[test]
    /// tests keys.public_key()
    fn keys_public_key() {
        assert_eq!(test_keys().public_key(), test_public_key());
    }

    #[test]
    /// tests keys.private_key()
    fn keys_private_key() {
        assert_eq!(test_keys().private_key(), test_private_key());
    }

}
