#[derive(Clone, Debug, PartialEq)]
pub struct Key {}

impl Key {

    pub fn new() -> Key {
        Key{}
    }

}

#[derive(Clone, Debug, PartialEq)]
pub struct Keys {
    pub_key: Key,
    priv_key: Key,
    node_id: String,
}

impl Keys {

    pub fn new<S: Into<String>> (pub_key: &Key, priv_key: &Key, node_id: S) -> Keys {
        Keys {
            pub_key: pub_key.clone(),
            priv_key: priv_key.clone(),
            node_id: node_id.into(),
        }
    }

    /// getter for the public key
    pub fn pub_key(&self) -> Key {
        self.pub_key.clone()
    }

    /// getter for the private key
    pub fn priv_key(&self) -> Key {
        self.priv_key.clone()
    }

    /// getter for the node id
    pub fn node_id(&self) -> String {
        self.node_id.clone()
    }
}

#[cfg(test)]
pub mod tests {

    use super::Key;
    use super::Keys;

    /// generates a new key suitable for testing
    pub fn test_key() -> Key {
        Key::new()
    }

    /// generates a new node id suitable for testing
    pub fn test_node_id() -> String {
        "test node id".into()
    }

    /// generates new id/pub/priv keys suitable for testing
    pub fn test_keys() -> Keys {
        Keys::new(&test_key(), &test_key(), test_node_id())
    }

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

}
