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
}
