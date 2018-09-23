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

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct EntityAttributeValue {
    entity: Entity,
    attribute: Attribute,
    value: Value,
    // index: Index,
    // source: Source,
}

impl EntityAttributeValue {
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
    fn add_eav(&mut self, eav: EntityAttributeValue) -> Result<(), HolochainError>;
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

    pub struct ExampleEntityAttributeValueStorage {
        eavs: HashSet<EntityAttributeValue>,
    }

    impl EntityAttributeValueStorage for ExampleEntityAttributeValueStorage {
        fn add_eav(&mut self, eav: EntityAttributeValue) -> Result<(), HolochainError> {
            self.eavs.insert(eav);
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

    }
}
