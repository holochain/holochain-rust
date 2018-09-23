use error::HolochainError;
use cas::content::Address;
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

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct EntityAttributeValue {
    entity: Entity,
    attribute: Attribute,
    value: Value,
    // index: Index,
    // source: Source,
}

impl EntityAttributeValue {
    pub fn new(entity: &Entity, attribute: &Attribute, value: &Value) -> EntityAttributeValue {
        EntityAttributeValue{
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
    fn fetch_eav(&self, entity: Option<Entity>, attribute: Option<Attribute>, value: Option<Value>) -> Result<HashSet<EntityAttributeValue>, HolochainError>;
}

#[cfg(test)]
pub mod tests {
    use error::HolochainError;
    use cas::eav::Entity;
    use cas::eav::Attribute;
    use cas::eav::Value;
    use cas::eav::EntityAttributeValueStorage;
    use cas::eav::EntityAttributeValue;
    use std::collections::HashSet;
    use cas::content::Content;
    use cas::content::AddressableContent;
    use cas::content::tests::ExampleAddressableContent;

    pub struct ExampleEntityAttributeValueStorage {
        eavs: HashSet<EntityAttributeValue>,
    }

    impl ExampleEntityAttributeValueStorage {
        pub fn new() -> ExampleEntityAttributeValueStorage {
            ExampleEntityAttributeValueStorage{
                eavs: HashSet::new(),
            }
        }
    }

    impl EntityAttributeValueStorage for ExampleEntityAttributeValueStorage {
        fn add_eav(&mut self, eav: &EntityAttributeValue) -> Result<(), HolochainError> {
            self.eavs.insert(eav.clone());
            Ok(())
        }

        fn fetch_eav(&self, entity: Option<Entity>, attribute: Option<Attribute>, value: Option<Value>) -> Result<HashSet<EntityAttributeValue>, HolochainError> {
            let filtered = self.eavs.iter().cloned()
                    .filter(|eav| {
                        match entity {
                            Some(ref e) => &eav.entity() == e,
                            None => true,
                        }
                    })
                    .filter(|eav| {
                        match attribute {
                            Some(ref a) => &eav.attribute() == a,
                            None => true,
                        }
                    })
                    .filter(|eav| {
                        match value {
                            Some(ref v) => &eav.value() == v,
                            None => true,
                        }
                    })
                    .collect::<HashSet<EntityAttributeValue>>();
            Ok(filtered)
        }
    }

    #[test]
    fn example_eav_round_trip () {
        let entity_content = ExampleAddressableContent::from_content(&"foo".to_string());
        let attribute = "favourite-color".to_string();
        let value_content: Content = AddressableContent::from_content(&"blue".to_string());

        let eav = EntityAttributeValue::new(&entity_content.address(), &attribute, &value_content.address());
        let mut eav_storage = ExampleEntityAttributeValueStorage::new();

        assert_eq!(
            HashSet::new(),
            eav_storage
                .fetch_eav(Some(entity_content.address()), Some(attribute.clone()), Some(value_content.address()))
                .expect("could not fetch eav"),
        );

        eav_storage.add_eav(&eav).expect("could not add eav");

        let mut expected = HashSet::new();
        expected.insert(eav.clone());
        // some examples of constraints that should all return the eav
        for (e, a, v) in vec![// constrain all
                              (Some(entity_content.address()), Some(attribute.clone()), Some(value_content.address())),
                              // open entity
                              (None, Some(attribute.clone()), Some(value_content.address())),
                              // open attribute
                              (Some(entity_content.address()), None, Some(value_content.address())),
                              // open value
                              (Some(entity_content.address()), Some(attribute.clone()), None),
                              // open
                              (None, None, None),
        ] {
            assert_eq!(
                expected,
                eav_storage
                    .fetch_eav(e, a, v)
                    .expect("could not fetch eav"),
            );
        }
    }
}
