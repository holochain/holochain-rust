use cas::content::{Address, AddressableContent};
use error::HolochainError;

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
    /// @see the fetch implementation for ExampleCas in the cas module tests
    fn fetch<C: AddressableContent>(&self, address: &Address) -> Result<Option<C>, HolochainError>;
}

#[cfg(test)]
pub mod tests {
    use cas::{
        content::{Address, AddressableContent, Content},
        storage::ContentAddressableStorage,
    };
    use error::HolochainError;
    use std::collections::HashMap;
    use cas::content::tests::{ExampleAddressableContent, OtherExampleAddressableContent};

    /// some struct to show an example ContentAddressableStorage implementation
    /// there is no persistence or concurrency in this example so use a raw HashMap
    pub struct ExampleContentAddressableStorage {
        storage: HashMap<Address, Content>,
    }

    impl ExampleContentAddressableStorage {
        pub fn new() -> ExampleContentAddressableStorage {
            ExampleContentAddressableStorage {
                storage: HashMap::new(),
            }
        }
    }

    impl ContentAddressableStorage for ExampleContentAddressableStorage {
        fn add(&mut self, content: &AddressableContent) -> Result<(), HolochainError> {
            self.storage.insert(content.address(), content.content());
            Ok(())
        }

        fn contains(&self, address: &Address) -> Result<bool, HolochainError> {
            Ok(self.storage.contains_key(address))
        }

        fn fetch<C: AddressableContent>(
            &self,
            address: &Address,
        ) -> Result<Option<C>, HolochainError> {
            Ok(self
                .storage
                .get(address)
                .and_then(|c| Some(C::from_content(c))))
        }
    }

    #[test]
    /// show that content of different types can round trip through the same storage
    fn example_content_round_trip_test() {
        let content = ExampleAddressableContent::from_content(&"foo".to_string());
        let other_content = OtherExampleAddressableContent::from_content(&"bar".to_string());
        let mut cas = ExampleContentAddressableStorage::new();

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
        assert_eq!(
            Ok(Some(other_content.clone())),
            cas.fetch(&other_content.address())
        );
    }
}
