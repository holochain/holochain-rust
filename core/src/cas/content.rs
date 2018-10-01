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
    use cas::content::{Address, AddressableContent, Content};
    use hash::HashString;
    use multihash::Hash;

    #[derive(Debug, PartialEq, Clone, Hash, Eq)]
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
        address: Address,
    }

    /// address is calculated eagerly rather than on call
    impl AddressableContent for OtherExampleAddressableContent {
        fn address(&self) -> Address {
            self.address.clone()
        }

        fn content(&self) -> Content {
            self.content.clone()
        }

        fn from_content(content: &Content) -> Self {
            OtherExampleAddressableContent {
                content: content.clone(),
                address: HashString::encode_from_str(&content, Hash::SHA2256),
            }
        }
    }

    /// fake content for addressable content examples
    pub fn test_content() -> Content {
        "foo".to_string()
    }

    /// fake ExampleAddressableContent
    pub fn test_example_addressable_content() -> ExampleAddressableContent {
        ExampleAddressableContent::from_content(&test_content())
    }

    /// fake OtherExampleAddressableContent
    pub fn test_other_example_addressable_content() -> OtherExampleAddressableContent {
        OtherExampleAddressableContent::from_content(&test_content())
    }

    #[test]
    /// test the first example
    fn example_addressable_content_trait_test() {
        let example_addressable_content = test_example_addressable_content();

        assert_eq!(
            example_addressable_content,
            ExampleAddressableContent::from_content(&test_content())
        );
        assert_eq!(test_content(), example_addressable_content.content());
        assert_eq!(
            HashString::from("QmRJzsvyCQyizr73Gmms8ZRtvNxmgqumxc2KUp71dfEmoj".to_string()),
            example_addressable_content.address()
        );
    }

    #[test]
    /// test the other example
    fn other_example_addressable_content_trait_test() {
        let other_example_addressable_content = test_other_example_addressable_content();

        assert_eq!(
            other_example_addressable_content,
            OtherExampleAddressableContent::from_content(&test_content())
        );
        assert_eq!(test_content(), other_example_addressable_content.content());
        assert_eq!(
            HashString::from("QmRJzsvyCQyizr73Gmms8ZRtvNxmgqumxc2KUp71dfEmoj".to_string()),
            other_example_addressable_content.address()
        );
    }

    #[test]
    /// test that both implementations do the same thing
    fn example_addressable_contents_are_the_same_test() {
        let example_addressable_content = test_example_addressable_content();
        let other_example_addressable_content = test_other_example_addressable_content();

        assert_eq!(
            example_addressable_content.content(),
            other_example_addressable_content.content()
        );
        assert_eq!(
            example_addressable_content.address(),
            other_example_addressable_content.address()
        );
    }

}
