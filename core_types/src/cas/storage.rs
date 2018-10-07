use cas::content::{Address, AddressableContent, Content};
use error::HolochainError;
use std::{collections::HashMap, fmt::Debug};
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

    fn fetch<C: AddressableContent>(&self, address: &Address) -> Result<Option<C>, HolochainError> {
        Ok(self
            .storage
            .get(address)
            .and_then(|c| Some(C::from_content(c))))
    }
}

//A struct for our test suite that infers a type of ContentAddressableStorage
pub struct StorageTestSuite<T>
where
    T: ContentAddressableStorage,
{
    cas: T,
}

impl<T> StorageTestSuite<T>
where
    T: ContentAddressableStorage,
{
    pub fn new(cas: T) -> StorageTestSuite<T> {
        StorageTestSuite { cas: cas }
    }

    //does round trip test that can infer two Addressable Content Types
    pub fn round_trip_test<Addressable, OtherAddressable>(
        mut self,
        content: Content,
        other_content: Content,
    ) where
        Addressable: AddressableContent + Clone + PartialEq + Debug,
        OtherAddressable: AddressableContent + Clone + PartialEq + Debug,
    {
        //based on associate type we call the right from_content function
        let addressable_content = Addressable::from_content(&content);
        let other_addressable_content = OtherAddressable::from_content(&other_content);

        assert_eq!(Ok(false), self.cas.contains(&addressable_content.address()));
        assert_eq!(
            Ok(None),
            self.cas
                .fetch::<Addressable>(&addressable_content.address())
        );

        assert_eq!(
            Ok(false),
            self.cas.contains(&other_addressable_content.address())
        );
        assert_eq!(
            Ok(None),
            self.cas
                .fetch::<OtherAddressable>(&other_addressable_content.address())
        );

        // round trip some AddressableContent through the ContentAddressableStorage
        assert_eq!(Ok(()), self.cas.add(&content));
        assert_eq!(Ok(true), self.cas.contains(&content.address()));
        assert_eq!(Ok(false), self.cas.contains(&other_content.address()));
        assert_eq!(
            Ok(Some(content.clone())),
            self.cas.fetch(&content.address())
        );

        // multiple types of AddressableContent can sit in a single ContentAddressableStorage
        // the safety of this is only as good as the hashing algorithm(s) used
        assert_eq!(Ok(()), self.cas.add(&other_content));
        assert_eq!(Ok(true), self.cas.contains(&content.address()));
        assert_eq!(Ok(true), self.cas.contains(&other_content.address()));
        assert_eq!(
            Ok(Some(content.clone())),
            self.cas.fetch(&content.address())
        );
        assert_eq!(
            Ok(Some(other_content.clone())),
            self.cas.fetch(&other_content.address())
        );
    }
}

#[cfg(test)]
pub mod tests {
    use cas::{
        content::{ExampleAddressableContent, OtherExampleAddressableContent},
        storage::{ExampleContentAddressableStorage, StorageTestSuite},
    };

    /// show that content of different types can round trip through the same storage
    #[test]
    fn example_content_round_trip_test() {
        let test_suite = StorageTestSuite::new(ExampleContentAddressableStorage::new());
        test_suite.round_trip_test::<ExampleAddressableContent, OtherExampleAddressableContent>(
            String::from("foo"),
            String::from("bar"),
        );
    }
}
