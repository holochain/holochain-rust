use hash::HashString;
use multihash::Hash;

/// an Address for some Content
/// ideally would be the Content but pragmatically must be HashString
/// consider what would happen if we had multi GB addresses...
pub type Address = HashString;
/// the Content is a String
/// this is the only way to be confident in persisting all Rust types across all backends
pub type Content = String;

/// can be stored as serialized content
/// the content is the address, there is no "location" like a file system or URL
/// @see https://en.wikipedia.org/wiki/Content-addressable_storage
pub trait AddressableContent {
    /// the Address the Content would be available at once stored in a ContentAddressableStorage
    /// default implementation is provided as hashing Content with sha256
    /// the default implementation should cover most use-cases
    /// it is critical that there are no hash collisions across all stored AddressableContent
    /// it is recommended to implement an "address space" prefix for address algorithms that don't
    /// offer strong cryptographic guarantees like sha et. al.
    fn address(&self) -> Address {
        HashString::encode_from_str(&self.content(), Hash::SHA2256)
    }
    /// the Content that would be stored in a ContentAddressableStorage
    fn content(&self) -> Content;
    /// restore/deserialize the original struct/type from serialized Content
    fn from_content(&Content) -> Self
    where
        Self: Sized;
}

impl AddressableContent for Content {
    fn content(&self) -> Content {
        self.clone()
    }

    fn from_content(content: &Content) -> Self {
        content.clone()
    }
}

#[cfg(test)]
pub mod tests {
    use cas::content::{AddressableContent, Content};

    #[derive(Debug, PartialEq, Clone)]
    /// some struct that can be content addressed
    /// imagine an Entry, Header, Meta Value, etc.
    pub struct ExampleAddressableContent {
        content: Content,
    }

    impl AddressableContent for ExampleAddressableContent {
        fn content(&self) -> Content {
            self.content.clone()
        }

        fn from_content(content: &Content) -> Self {
            ExampleAddressableContent {
                content: content.clone(),
            }
        }
    }

    #[derive(Debug, PartialEq, Clone)]
    /// another struct that can be content addressed
    /// used to show ExampleCas storing multiple types
    pub struct OtherExampleAddressableContent {
        content: Content,
    }

    /// identical implementation to ExampleAddressableContent for simplicity
    impl AddressableContent for OtherExampleAddressableContent {
        fn content(&self) -> Content {
            self.content.clone()
        }

        fn from_content(content: &Content) -> Self {
            OtherExampleAddressableContent {
                content: content.clone(),
            }
        }
    }

}
