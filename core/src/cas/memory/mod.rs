use cas::{
    content::{Address, AddressableContent, Content},
    storage::ContentAddressableStorage,
};
use error::HolochainError;
use std::collections::HashMap;

pub struct MemoryStorage {
    storage: HashMap<Address, Content>,
}

impl MemoryStorage {
    fn new() -> MemoryStorage {
        MemoryStorage {
            storage: HashMap::new(),
        }
    }
}

impl ContentAddressableStorage for MemoryStorage {
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

#[cfg(test)]
pub mod tests {
    use cas::{
        content::{
            tests::{ExampleAddressableContent, OtherExampleAddressableContent},
            AddressableContent,
        },
        memory::MemoryStorage,
        storage::ContentAddressableStorage,
    };
    use tempfile::{tempdir, TempDir};

    #[test]
    fn memory_round_trip() {
        let mut cas = MemoryStorage::new();
        let content = ExampleAddressableContent::from_content(&"foo".to_string());
        let other_content = OtherExampleAddressableContent::from_content(&"bar".to_string());
        assert_eq!(Ok(false), cas.contains(&content.address()));
        assert_eq!(
            Ok(None),
            cas.fetch::<ExampleAddressableContent>(&content.address())
        );
        assert_eq!(Ok(false), cas.contains(&other_content.address()));
        assert_eq!(
            Ok(None),
            cas.fetch::<OtherExampleAddressableContent>(&other_content.address())
        );
        // round trip some AddressableContent through the MemoryStorage
        assert_eq!(Ok(()), cas.add(&content));
        assert_eq!(Ok(true), cas.contains(&content.address()));
        assert_eq!(Ok(false), cas.contains(&other_content.address()));
        assert_eq!(Ok(Some(content.clone())), cas.fetch(&content.address()));
        // multiple types of AddressableContent can sit in a single MemoryStorage
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
