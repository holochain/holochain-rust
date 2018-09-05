pub trait Key {
    /// returns the key for self that can be used in key/value contexts
    fn key(&self) -> String;
}
