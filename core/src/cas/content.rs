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
    use cas::{
        content::{Address, AddressableContent, Content},
        storage::ContentAddressableStorage,
    };
    use hash::HashString;
    use multihash::Hash;
    use std::fmt::{Debug, Write};

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

    pub struct AddressableContentTestSuite;

    impl AddressableContentTestSuite {
        /// test that trait gives the write content
        pub fn addressable_content_trait_test<T>(
            content: Content,
            expected_content: T,
            hash_string: String,
        ) where
            T: AddressableContent + Debug + PartialEq + Clone,
        {
            let addressable_content = T::from_content(&content);

            assert_eq!(addressable_content, expected_content);
            assert_eq!(content, addressable_content.content());
            assert_eq!(HashString::from(hash_string), addressable_content.address());
        }

        /// test that two different addressable contents would give them same thing
        pub fn addressable_contents_are_the_same_test<T, K>(content: Content)
        where
            T: AddressableContent + Debug + PartialEq + Clone,
            K: AddressableContent + Debug + PartialEq + Clone,
        {
            let addressable_content = T::from_content(&content);
            let other_addressable_content = K::from_content(&content);

            assert_eq!(
                addressable_content.content(),
                other_addressable_content.content()
            );
            assert_eq!(
                addressable_content.address(),
                other_addressable_content.address()
            );
        }

        pub fn addressalbe_content_round_trip<T, K>(contents: Vec<T>, mut cas: K)
        where
            T: AddressableContent + PartialEq + Clone + Debug,
            K: ContentAddressableStorage,
        {
            contents.into_iter().for_each(|f| {
                let mut add_error_message = String::new();
                let mut fetch_error_message = String::new();
                writeln!(&mut add_error_message, "Could not add {:?}", f.clone());
                writeln!(&mut fetch_error_message, "Could not fetch {:?}", f.clone());

                cas.add(&f).expect(&add_error_message);
                assert_eq!(
                    Some(f.clone()),
                    cas.fetch::<T>(&f.address()).expect(&fetch_error_message)
                );
            });
        }
    }

    #[test]
    /// test the first example
    fn example_addressable_content_trait_test() {
        AddressableContentTestSuite::addressable_content_trait_test::<ExampleAddressableContent>(
            String::from("foo"),
            ExampleAddressableContent::from_content(&String::from("foo")),
            String::from("QmRJzsvyCQyizr73Gmms8ZRtvNxmgqumxc2KUp71dfEmoj"),
        );
    }

    #[test]
    /// test the other example
    fn other_example_addressable_content_trait_test() {
        AddressableContentTestSuite::addressable_content_trait_test::<OtherExampleAddressableContent>(
            String::from("foo"),
            OtherExampleAddressableContent::from_content(&String::from("foo")),
            String::from("QmRJzsvyCQyizr73Gmms8ZRtvNxmgqumxc2KUp71dfEmoj"),
        );
    }

    #[test]
    /// test that both implementations do the same thing
    fn example_addressable_contents_are_the_same_test() {
        AddressableContentTestSuite::addressable_contents_are_the_same_test::<
            ExampleAddressableContent,
            OtherExampleAddressableContent,
        >(String::from("foo"));
    }

}
