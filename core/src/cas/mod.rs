use error::HolochainError;
use hash::HashString;

/// an address for some Content
/// ideally it would be the Content but pragmatically it must be some HashString
/// consider what would happen if we had multi GB addresses...
type Address = HashString;
/// the Content is a String
/// this is the only way to be confident in persisting all Rust types across all backends
type Content = String;

/// can be stored as serialized content
/// the content is the address, there is no "location" like a file system or URL
/// @see https://en.wikipedia.org/wiki/Content-addressable_storage
pub trait AddressableContent {
    fn address(&self) -> Address;
    fn content(&self) -> Content;
}

/// content addressable store
/// implements storage in memory or persistently
/// anything implementing AddressableContent can be stored and retrieved
pub trait ContentAddressableStorage {
    /// stores AddressableContent in the Store by its Address as Content
    fn add(&self, content: &AddressableContent) -> Result<(), HolochainError>;
    /// true if the Address is in the Store, false otherwise.
    /// may be more efficient than retrieve depending on the implementation.
    fn contains(&self, address: &Address) -> Result<bool, HolochainError>;
    /// returns some Content String if it is in the Store
    /// note: the original struct/type is NOT restored/deserialized
    fn fetch(&self, address: &Address) -> Result<Option<Content>, HolochainError>;
}
