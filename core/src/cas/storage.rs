use cas::content::{Address, AddressableContent};
use error::HolochainError;

/// content addressable store (CAS)
/// implements storage in memory or persistently
/// anything implementing AddressableContent can be added and fetched by address
/// CAS is append only
pub trait ContentAddressableStorage: Clone {
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
    use actor::{AskSelf, Protocol, SYS};
    use cas::{
        content::{
            tests::{ExampleAddressableContent, OtherExampleAddressableContent},
            Address, AddressableContent, Content,
        },
        storage::ContentAddressableStorage,
    };
    use error::HolochainError;
    use riker::actors::*;
    use snowflake;
    use std::{collections::HashMap, fmt::Debug};

    #[derive(Clone)]
    /// some struct to show an example ContentAddressableStorage implementation
    /// there is no persistence or concurrency in this example so use a raw HashMap
    /// @see ExampleContentAddressableStorageActor
    pub struct ExampleContentAddressableStorage {
        actor: ActorRef<Protocol>,
    }

    impl ExampleContentAddressableStorage {
        pub fn new() -> Result<ExampleContentAddressableStorage, HolochainError> {
            Ok(ExampleContentAddressableStorage {
                actor: ExampleContentAddressableStorageActor::new_ref()?,
            })
        }
    }

    /// show an example Actor for ContentAddressableStorage
    /// a key requirement of the CAS is that cloning doesn't undermine data consistency
    /// a key requirement of the CAS is that multithreading doesn't undermine data consistency
    /// actors deliver on both points through the ActorRef<Protocol> abstraction
    /// cloned actor references point to the same actor with the same internal state
    /// actors have internal message queues to co-ordinate requests
    /// the tradeoff is boilerplate + some overhead from the actor system
    pub struct ExampleContentAddressableStorageActor {
        storage: HashMap<Address, Content>,
    }

    impl ExampleContentAddressableStorageActor {
        pub fn new() -> ExampleContentAddressableStorageActor {
            ExampleContentAddressableStorageActor {
                storage: HashMap::new(),
            }
        }

        fn actor() -> BoxActor<Protocol> {
            Box::new(ExampleContentAddressableStorageActor::new())
        }

        fn props() -> BoxActorProd<Protocol> {
            Props::new(Box::new(ExampleContentAddressableStorageActor::actor))
        }

        pub fn new_ref() -> Result<ActorRef<Protocol>, HolochainError> {
            Ok(SYS.actor_of(
                ExampleContentAddressableStorageActor::props(),
                // all actors have the same ID to allow round trip across clones
                &snowflake::ProcessUniqueId::new().to_string(),
            )?)
        }

        fn unthreadable_add(
            &mut self,
            address: &Address,
            content: &Content,
        ) -> Result<(), HolochainError> {
            self.storage.insert(address.clone(), content.clone());
            Ok(())
        }

        fn unthreadable_contains(&self, address: &Address) -> Result<bool, HolochainError> {
            Ok(self.storage.contains_key(address))
        }

        fn unthreadable_fetch(&self, address: &Address) -> Result<Option<Content>, HolochainError> {
            Ok(self.storage.get(address).cloned())
        }
    }

    /// this is all boilerplate
    impl Actor for ExampleContentAddressableStorageActor {
        type Msg = Protocol;

        fn receive(
            &mut self,
            context: &Context<Self::Msg>,
            message: Self::Msg,
            sender: Option<ActorRef<Self::Msg>>,
        ) {
            sender
                .try_tell(
                    match message {
                        Protocol::CasAdd(address, content) => {
                            Protocol::CasAddResult(self.unthreadable_add(&address, &content))
                        }
                        Protocol::CasContains(address) => {
                            Protocol::CasContainsResult(self.unthreadable_contains(&address))
                        }
                        Protocol::CasFetch(address) => {
                            Protocol::CasFetchResult(self.unthreadable_fetch(&address))
                        }
                        _ => unreachable!(),
                    },
                    Some(context.myself()),
                )
                .expect("failed to tell MemoryStorageActor sender");
        }
    }

    impl ContentAddressableStorage for ExampleContentAddressableStorage {
        fn add(&mut self, content: &AddressableContent) -> Result<(), HolochainError> {
            let response = self
                .actor
                .block_on_ask(Protocol::CasAdd(content.address(), content.content()))?;
            unwrap_to!(response => Protocol::CasAddResult).clone()
        }

        fn contains(&self, address: &Address) -> Result<bool, HolochainError> {
            let response = self
                .actor
                .block_on_ask(Protocol::CasContains(address.clone()))?;
            unwrap_to!(response => Protocol::CasContainsResult).clone()
        }

        fn fetch<AC: AddressableContent>(
            &self,
            address: &Address,
        ) -> Result<Option<AC>, HolochainError> {
            let response = self
                .actor
                .block_on_ask(Protocol::CasFetch(address.clone()))?;
            let content = unwrap_to!(response => Protocol::CasFetchResult).clone()?;
            Ok(match content {
                Some(c) => Some(AC::from_content(&c)),
                None => None,
            })
        }
    }

    pub fn test_content_addressable_storage() -> ExampleContentAddressableStorage {
        ExampleContentAddressableStorage::new().expect("could not build example cas")
    }

    //A struct for our test suite that infers a type of ContentAddressableStorage
    pub struct StorageTestSuite<T>
    where
        T: ContentAddressableStorage,
    {
        cas: T,
        /// it is important that every cloned copy of any CAS has a consistent view to data
        cas_clone: T,
    }

    impl<T> StorageTestSuite<T>
    where
        T: ContentAddressableStorage,
    {
        pub fn new(cas: T) -> StorageTestSuite<T> {
            StorageTestSuite {
                cas_clone: cas.clone(),
                cas: cas,
            }
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
            // based on associate type we call the right from_content function
            let addressable_content = Addressable::from_content(&content);
            let other_addressable_content = OtherAddressable::from_content(&other_content);

            // do things that would definitely break if cloning would show inconsistent data
            let both_cas = vec![self.cas.clone(), self.cas_clone.clone()];

            for cas in both_cas.iter() {
                assert_eq!(Ok(false), cas.contains(&addressable_content.address()));
                assert_eq!(
                    Ok(None),
                    cas.fetch::<Addressable>(&addressable_content.address())
                );
                assert_eq!(
                    Ok(false),
                    cas.contains(&other_addressable_content.address())
                );
                assert_eq!(
                    Ok(None),
                    cas.fetch::<OtherAddressable>(&other_addressable_content.address())
                );
            }

            // round trip some AddressableContent through the ContentAddressableStorage
            assert_eq!(Ok(()), self.cas.add(&content));

            for cas in both_cas.iter() {
                assert_eq!(Ok(true), cas.contains(&content.address()));
                assert_eq!(Ok(false), cas.contains(&other_content.address()));
                assert_eq!(Ok(Some(content.clone())), cas.fetch(&content.address()));
            }

            // multiple types of AddressableContent can sit in a single ContentAddressableStorage
            // the safety of this is only as good as the hashing algorithm(s) used
            assert_eq!(Ok(()), self.cas_clone.add(&other_content));

            for cas in both_cas.iter() {
                assert_eq!(Ok(true), cas.contains(&content.address()));
                assert_eq!(Ok(true), cas.contains(&other_content.address()));
                assert_eq!(Ok(Some(content.clone())), cas.fetch(&content.address()));
                assert_eq!(
                    Ok(Some(other_content.clone())),
                    cas.fetch(&other_content.address())
                );
            }
        }
    }

    /// show that content of different types can round trip through the same storage
    #[test]
    fn example_content_round_trip_test() {
        let test_suite = StorageTestSuite::new(test_content_addressable_storage());
        test_suite.round_trip_test::<ExampleAddressableContent, OtherExampleAddressableContent>(
            String::from("foo"),
            String::from("bar"),
        );
    }
}
