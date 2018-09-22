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
    fn address(&self) -> Address;
    fn content(&self) -> Content;
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
    /// returns Some Content String if it is in the Store, else None
    /// note: the original struct/type is NOT restored/deserialized
    fn fetch(&self, address: &Address) -> Result<Option<Content>, HolochainError>;
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
    }

    impl ExampleAddressableContent {
        fn new(s: &str) -> ExampleAddressableContent {
            ExampleAddressableContent {
                content: s.to_string(),
            }
        }
    }

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

        fn fetch(&self, address: &Address) -> Result<Option<Content>, HolochainError> {
            Ok(self.storage.get(address).and_then(|c| Some(c.to_string())))
        }
    }

    #[test]
    fn example_round_trip() {
        let content = ExampleAddressableContent::new("foo");
        let mut cas = ExampleCas::new();

        assert_eq!(Ok(false), cas.contains(&content.address()));

        assert_eq!(Ok(()), cas.add(&content));
        assert_eq!(Ok(true), cas.contains(&content.address()));
        assert_eq!(Ok(Some(content.content())), cas.fetch(&content.address()));
    }
}
