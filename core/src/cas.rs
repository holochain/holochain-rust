use error::HolochainError;
use hash::HashString;

/// an Address for some Content
/// ideally would be the Content but pragmatically must be HashString
/// consider what would happen if we had multi GB addresses...
type Address = HashString;
/// the Content is a String
/// this is the only way to be confident in persisting all Rust types across all backends
type Content = String;

/// can be stored as serialized content
/// the content is the address, there is no "location" like a file system or URL
/// @see https://en.wikipedia.org/wiki/Content-addressable_storage
pub trait AddressableContent {
    /// the Address the Content would be available at once stored in a ContentAddressableStorage
    fn address(&self) -> Address;
    /// the Content that would be stored in a ContentAddressableStorage
    fn content(&self) -> Content;
    /// restore/deserialize the original struct/type from serialized Content
    fn from_content(&Content) -> Self where Self: Sized;
}

/// content addressable store (CAS)
/// implements storage in memory or persistently
/// anything implementing AddressableContent can be added and fetched by address
/// CAS is append only
pub trait ContentAddressableStorage {
    /// adds AddressableContent to the ContentAddressableStorage by its Address as Content
    fn add(&mut self, content: &AddressableContent) -> Result<(), HolochainError>;
    /// true if the Address is in the Store, false otherwise.
    /// may be more efficient than retrieve depending on the implementation.
    fn contains(&self, address: &Address) -> Result<bool, HolochainError>;
    /// returns Some AddressableContent if it is in the Store, else None
    /// AddressableContent::from_content() can be used to allow the compiler to infer the type
    /// @see the fetch implementation for ExampleCas in the tests below
    fn fetch<C: AddressableContent> (&self, address: &Address) -> Result<Option<C>, HolochainError>;
}

#[cfg(test)]
mod tests {
    use cas::ContentAddressableStorage;
    use cas::AddressableContent;
    use std::collections::HashMap;
    use cas::Address;
    use cas::Content;
    use error::HolochainError;
    use hash::HashString;
    use multihash::Hash;

    #[derive(Debug, PartialEq, Clone)]
    /// some struct that can be content addressed
    /// imagine an Entry, Header, MetaValue, etc.
    struct ExampleAddressableContent {
        content: Content,
    }

    impl AddressableContent for ExampleAddressableContent {
        fn address (&self) -> Address {
            HashString::encode_from_str(&self.content(), Hash::SHA2256)
        }

        fn content (&self) -> Content {
            self.content.clone()
        }

        fn from_content (content: &Content) -> Self {
            ExampleAddressableContent {
                content: content.clone(),
            }
        }
    }

    #[derive(Debug, PartialEq, Clone)]
    /// another struct that can be content addressed
    /// used to show ExampleCas storing multiple types
    struct OtherExampleAddressableContent {
        content: Content,
    }

    /// identical implementation to ExampleAddressableContent for simplicity
    impl AddressableContent for OtherExampleAddressableContent {
        fn address (&self) -> Address {
            HashString::encode_from_str(&self.content(), Hash::SHA2256)
        }

        fn content (&self) -> Content {
            self.content.clone()
        }

        fn from_content (content: &Content) -> Self {
            OtherExampleAddressableContent {
                content: content.clone(),
            }
        }
    }

    /// some struct to show an example ContentAddressableStorage implementation
    /// there is no persistence or concurrency in this example so use a raw HashMap
    struct ExampleCas {
        storage: HashMap<Address, Content>,
    }

    impl ExampleCas {
        fn new() -> ExampleCas {
            ExampleCas{
                storage: HashMap::new(),
            }
        }
    }

    impl ContentAddressableStorage for ExampleCas {
        fn add(&mut self, content: &AddressableContent) -> Result<(), HolochainError> {
            self.storage.insert(content.address(), content.content());
            Ok(())
        }

        fn contains(&self, address: &Address) -> Result<bool, HolochainError> {
            Ok(self.storage.contains_key(address))
        }

        fn fetch<C: AddressableContent>(&self, address: &Address) -> Result<Option<C>, HolochainError> {
            Ok(self.storage.get(address).and_then(|c| Some(C::from_content(c))))
        }
    }

    #[test]
    fn example_round_trip() {
        let content = ExampleAddressableContent::from_content(&"foo".to_string());
        let other_content = OtherExampleAddressableContent::from_content(&"bar".to_string());
        let mut cas = ExampleCas::new();

        assert_eq!(Ok(false), cas.contains(&content.address()));
        assert_eq!(Ok(false), cas.contains(&other_content.address()));

        // round trip some AddressableContent through the ContentAddressableStorage
        assert_eq!(Ok(()), cas.add(&content));
        assert_eq!(Ok(true), cas.contains(&content.address()));
        assert_eq!(Ok(false), cas.contains(&other_content.address()));
        assert_eq!(Ok(Some(content.clone())), cas.fetch(&content.address()));

        // multiple types of AddressableContent can sit in a single ContentAddressableStorage
        // the safety of this is only as good as the hashing algorithm(s) used
        assert_eq!(Ok(()), cas.add(&other_content));
        assert_eq!(Ok(true), cas.contains(&content.address()));
        assert_eq!(Ok(true), cas.contains(&other_content.address()));
        assert_eq!(Ok(Some(content.clone())), cas.fetch(&content.address()));
        assert_eq!(Ok(Some(other_content.clone())), cas.fetch(&other_content.address()));
    }
}
