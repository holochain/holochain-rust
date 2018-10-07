pub mod memory;

use cas::content::{Address, AddressableContent, Content};
use error::HolochainError;
use serde_json;
use std::collections::HashSet;

/// EAV (entity-attribute-value) data
/// ostensibly for metadata about entries in the DHT
/// defines relationships between AddressableContent values
/// implemented on top of cas::storage::ContentAddressableStorage
/// @see https://en.wikipedia.org/wiki/Entity%E2%80%93attribute%E2%80%93value_model
/// Address of AddressableContent representing the EAV entity
type Entity = Address;

/// using String for EAV attributes (not e.g. an enum) keeps it simple and open
type Attribute = String;

/// Address of AddressableContent representing the EAV value
type Value = Address;

// @TODO do we need this?
// unique (local to the source) monotonically increasing number that can be used for crdt/ordering
// @see https://papers.radixdlt.com/tempo/#logical-clocks
// type Index ...

// @TODO do we need this?
// source agent asserting the meta
// type Source ...

#[derive(PartialEq, Eq, Hash, Clone, Debug, Serialize, Deserialize)]
pub struct EntityAttributeValue {
    entity: Entity,
    attribute: Attribute,
    value: Value,
    // index: Index,
    // source: Source,
}

impl AddressableContent for EntityAttributeValue {
    fn content(&self) -> Content {
        serde_json::to_string(self)
            .expect("could not serialize EntityAttributeValue to Json Content")
    }

    fn from_content(content: &Content) -> Self {
        serde_json::from_str(content)
            .expect("could not deserialize Json Content to EntityAttributeValue")
    }
}

impl EntityAttributeValue {
    pub fn new(entity: &Entity, attribute: &Attribute, value: &Value) -> EntityAttributeValue {
        EntityAttributeValue {
            entity: entity.clone(),
            attribute: attribute.clone(),
            value: value.clone(),
        }
    }

    pub fn entity(&self) -> Entity {
        self.entity.clone()
    }

    pub fn attribute(&self) -> Attribute {
        self.attribute.clone()
    }

    pub fn value(&self) -> Value {
        self.value.clone()
    }

    // this is a predicate for matching on eav values. Useful for reducing duplicated filtered code.
    pub fn filter_on_eav<T>(eav: T, e: &Option<T>) -> bool
    where
        T: PartialOrd,
    {
        match e {
            Some(ref a) => &eav == a,
            None => true,
        }
    }
}

/// eav storage
/// does NOT provide storage for AddressableContent
/// use cas::storage::ContentAddressableStorage to store AddressableContent
/// provides a simple and flexible interface to define relationships between AddressableContent
pub trait EntityAttributeValueStorage {
    /// adds the given EntityAttributeValue to the EntityAttributeValueStorage
    /// append only storage
    /// eavs are retrieved through constraint based lookups
    /// @see fetch_eav
    fn add_eav(&mut self, eav: &EntityAttributeValue) -> Result<(), HolochainError>;
    /// fetches the set of EntityAttributeValues that match constraints
    /// None = no constraint
    /// Some(Entity) = requires the given entity (e.g. all a/v pairs for the entity)
    /// Some(Attribute) = requires the given attribute (e.g. all links)
    /// Some(Value) = requires the given value (e.g. all entities referencing an Address)
    fn fetch_eav(
        &self,
        entity: Option<Entity>,
        attribute: Option<Attribute>,
        value: Option<Value>,
    ) -> Result<HashSet<EntityAttributeValue>, HolochainError>;
}

#[cfg(test)]
pub mod tests {
    use cas::{
        content::{
            tests::{AddressableContentTestSuite, ExampleAddressableContent},
            Address, AddressableContent, Content,
        },
        storage::tests::ExampleContentAddressableStorage,
    };
    use eav::{Attribute, Entity, EntityAttributeValue, EntityAttributeValueStorage, Value};
    use error::HolochainError;
    use hash_table::entry::{
        tests::{test_entry_a, test_entry_b},
        Entry,
    };
    use std::collections::HashSet;
    use cas::storage::tests::test_content_addressable_storage;

    pub struct ExampleEntityAttributeValueStorage {
        eavs: HashSet<EntityAttributeValue>,
    }

    impl ExampleEntityAttributeValueStorage {
        pub fn new() -> ExampleEntityAttributeValueStorage {
            ExampleEntityAttributeValueStorage {
                eavs: HashSet::new(),
            }
        }
    }

    impl EntityAttributeValueStorage for ExampleEntityAttributeValueStorage {
        fn add_eav(&mut self, eav: &EntityAttributeValue) -> Result<(), HolochainError> {
            self.eavs.insert(eav.clone());
            Ok(())
        }

        fn fetch_eav(
            &self,
            entity: Option<Entity>,
            attribute: Option<Attribute>,
            value: Option<Value>,
        ) -> Result<HashSet<EntityAttributeValue>, HolochainError> {
            let filtered = self
                .eavs
                .iter()
                .cloned()
                .filter(|eav| match entity {
                    Some(ref e) => &eav.entity() == e,
                    None => true,
                })
                .filter(|eav| match attribute {
                    Some(ref a) => &eav.attribute() == a,
                    None => true,
                })
                .filter(|eav| match value {
                    Some(ref v) => &eav.value() == v,
                    None => true,
                })
                .collect::<HashSet<EntityAttributeValue>>();
            Ok(filtered)
        }
    }

    pub fn test_eav_entity() -> Entry {
        test_entry_a()
    }

    pub fn test_eav_attribute() -> String {
        "foo:attribute".to_string()
    }

    pub fn test_eav_value() -> Entry {
        test_entry_b()
    }

    pub fn test_eav() -> EntityAttributeValue {
        EntityAttributeValue::new(
            &test_eav_entity().address(),
            &test_eav_attribute(),
            &test_eav_value().address(),
        )
    }

    pub fn test_eav_content() -> Content {
        test_eav().content()
    }

    pub fn test_eav_address() -> Address {
        test_eav().address()
    }

    pub fn eav_round_trip_test_runner(
        entity_content: impl AddressableContent,
        attribute: String,
        value_content: impl AddressableContent,
    ) {
        let eav = EntityAttributeValue::new(
            &entity_content.address(),
            &attribute,
            &value_content.address(),
        );
        let mut eav_storage = ExampleEntityAttributeValueStorage::new();

        assert_eq!(
            HashSet::new(),
            eav_storage
                .fetch_eav(
                    Some(entity_content.address()),
                    Some(attribute.clone()),
                    Some(value_content.address())
                )
                .expect("could not fetch eav"),
        );

        eav_storage.add_eav(&eav).expect("could not add eav");

        let mut expected = HashSet::new();
        expected.insert(eav.clone());
        // some examples of constraints that should all return the eav
        for (e, a, v) in vec![
            // constrain all
            (
                Some(entity_content.address()),
                Some(attribute.clone()),
                Some(value_content.address()),
            ),
            // open entity
            (None, Some(attribute.clone()), Some(value_content.address())),
            // open attribute
            (
                Some(entity_content.address()),
                None,
                Some(value_content.address()),
            ),
            // open value
            (
                Some(entity_content.address()),
                Some(attribute.clone()),
                None,
            ),
            // open
            (None, None, None),
        ] {
            assert_eq!(
                expected,
                eav_storage.fetch_eav(e, a, v).expect("could not fetch eav"),
            );
        }
    }

    #[test]
    fn example_eav_round_trip() {
        eav_round_trip_test_runner(
            ExampleAddressableContent::from_content(&"foo".to_string()),
            "favourite-color".to_string(),
            ExampleAddressableContent::from_content(&"blue".to_string()),
        );
    }

    #[test]
    fn example_eav_one_to_many() {
        let one = ExampleAddressableContent::from_content(&"foo".to_string());
        // it can reference itself, why not?
        let many_one = ExampleAddressableContent::from_content(&"foo".to_string());
        let many_two = ExampleAddressableContent::from_content(&"bar".to_string());
        let many_three = ExampleAddressableContent::from_content(&"baz".to_string());
        let attribute = "one_to_many".to_string();

        let mut eav_storage = ExampleEntityAttributeValueStorage::new();
        let mut expected = HashSet::new();
        for many in vec![many_one.clone(), many_two.clone(), many_three.clone()] {
            let eav = EntityAttributeValue::new(&one.address(), &attribute, &many.address());
            eav_storage.add_eav(&eav).expect("could not add eav");
            expected.insert(eav);
        }

        // throw an extra thing referencing many to show fetch ignores it
        let two = ExampleAddressableContent::from_content(&"foo".to_string());
        for many in vec![many_one.clone(), many_three.clone()] {
            eav_storage
                .add_eav(&EntityAttributeValue::new(
                    &two.address(),
                    &attribute,
                    &many.address(),
                ))
                .expect("could not add eav");
        }

        // show the many results for one
        assert_eq!(
            expected,
            eav_storage
                .fetch_eav(Some(one.address()), Some(attribute.clone()), None)
                .expect("could not fetch eav"),
        );

        // show one for the many results
        for many in vec![many_one.clone(), many_two.clone(), many_three.clone()] {
            let mut expected_one = HashSet::new();
            expected_one.insert(EntityAttributeValue::new(
                &one.address(),
                &attribute.clone(),
                &many.address(),
            ));
            assert_eq!(
                expected_one,
                eav_storage
                    .fetch_eav(None, Some(attribute.clone()), Some(many.address()))
                    .expect("could not fetch eav"),
            );
        }
    }

    #[test]
    fn example_eav_many_to_one() {
        let one = ExampleAddressableContent::from_content(&"foo".to_string());
        // it can reference itself, why not?
        let many_one = ExampleAddressableContent::from_content(&"foo".to_string());
        let many_two = ExampleAddressableContent::from_content(&"bar".to_string());
        let many_three = ExampleAddressableContent::from_content(&"baz".to_string());
        let attribute = "many_to_one".to_string();

        let mut eav_storage = ExampleEntityAttributeValueStorage::new();
        let mut expected = HashSet::new();
        for many in vec![many_one.clone(), many_two.clone(), many_three.clone()] {
            let eav = EntityAttributeValue::new(&many.address(), &attribute, &one.address());
            eav_storage.add_eav(&eav).expect("could not add eav");
            expected.insert(eav);
        }

        // throw an extra thing referenced by many to show fetch ignores it
        let two = ExampleAddressableContent::from_content(&"foo".to_string());
        for many in vec![many_one.clone(), many_three.clone()] {
            eav_storage
                .add_eav(&EntityAttributeValue::new(
                    &many.address(),
                    &attribute,
                    &two.address(),
                ))
                .expect("could not add eav");
        }

        // show the many referencing one
        assert_eq!(
            expected,
            eav_storage
                .fetch_eav(None, Some(attribute.clone()), Some(one.address()))
                .expect("could not fetch eav"),
        );

        // show one for the many results
        for many in vec![many_one.clone(), many_two.clone(), many_three.clone()] {
            let mut expected_one = HashSet::new();
            expected_one.insert(EntityAttributeValue::new(
                &many.address(),
                &attribute.clone(),
                &one.address(),
            ));
            assert_eq!(
                expected_one,
                eav_storage
                    .fetch_eav(Some(many.address()), Some(attribute.clone()), None)
                    .expect("could not fetch eav"),
            );
        }
    }

    #[test]
    /// show AddressableContent implementation
    fn addressable_content_test() {
        // from_content()
        AddressableContentTestSuite::addressable_content_trait_test::<EntityAttributeValue>(
            test_eav_content(),
            test_eav(),
            String::from(test_eav_address()),
        );
    }

    #[test]
    /// show CAS round trip
    fn cas_round_trip_test() {
        let addressable_contents = vec![test_eav()];
        AddressableContentTestSuite::addressable_content_round_trip::<
            EntityAttributeValue,
            ExampleContentAddressableStorage,
        >(addressable_contents, test_content_addressable_storage());
    }

}
