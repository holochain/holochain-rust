use crate::{
    cas::content::{Address, AddressableContent, Content},
    entry::{test_entry_a, test_entry_b, Entry},
    error::{HcResult, HolochainError},
    json::JsonString,
    hash::HashString,
};
use objekt;
use im::hashmap::HashMap;
use std::{
    collections::{HashSet},
    convert::TryInto,
    sync::{Arc, RwLock},
};

use regex::RegexBuilder;
use std::fmt::Debug;
/// EAV (entity-attribute-value) data
/// ostensibly for metadata about entries in the DHT
/// defines relationships between AddressableContent values
/// implemented on top of cas::storage::ContentAddressableStorage
/// @see https://en.wikipedia.org/wiki/Entity%E2%80%93attribute%E2%80%93value_model
/// Address of AddressableContent representing the EAV entity
pub type Entity = Address;

/// using String for EAV attributes (not e.g. an enum) keeps it simple and open
pub type Attribute = String;

/// Address of AddressableContent representing the EAV value
pub type Value = Address;

// @TODO do we need this?
// unique (local to the source) monotonically increasing number that can be used for crdt/ordering
// @see https://papers.radixdlt.com/tempo/#logical-clocks
// type Index ...

// @TODO do we need this?
// source agent asserting the meta
// type Source ...

#[derive(PartialEq, Eq, Hash, Clone, Debug, Serialize, Deserialize, DefaultJson)]
pub struct EntityAttributeValue {
    entity: Entity,
    attribute: Attribute,
    value: Value,
    // index: Index,
    // source: Source,
}

impl AddressableContent for EntityAttributeValue {
    fn content(&self) -> Content {
        self.to_owned().into()
    }

    fn try_from_content(content: &Content) -> Result<Self, HolochainError> {
        content.to_owned().try_into()
    }
}

fn validate_attribute(attribute: &Attribute) -> HcResult<()> {
    let regex = RegexBuilder::new(r#"[/:*?<>"'\\|+]"#)
        .build()
        .map_err(|_| HolochainError::ErrorGeneric("Could not create regex".to_string()))?;
    if !regex.is_match(attribute) {
        Ok(())
    } else {
        Err(HolochainError::ErrorGeneric(
            "Attribute name invalid".to_string(),
        ))
    }
}

impl EntityAttributeValue {
    pub fn new(
        entity: &Entity,
        attribute: &Attribute,
        value: &Value,
    ) -> HcResult<EntityAttributeValue> {
        validate_attribute(attribute)?;
        Ok(EntityAttributeValue {
            entity: entity.clone(),
            attribute: attribute.clone(),
            value: value.clone(),
        })
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

    /// this is a predicate for matching on eav values. Useful for reducing duplicated filtered code.
    pub fn filter_on_eav<T>(eav: &T, e: Option<&T>) -> bool
    where
        T: PartialOrd,
    {
        e.map_or(true, |a| eav == a)
    }
}

/// eav storage
/// does NOT provide storage for AddressableContent
/// use cas::storage::ContentAddressableStorage to store AddressableContent
/// provides a simple and flexible interface to define relationships between AddressableContent
pub trait EntityAttributeValueStorage: objekt::Clone + Send + Sync + Debug {
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
    ) -> Result<HashMap<HashString,EntityAttributeValue>, HolochainError>;

    fn get_hash(&self)->HashString;
}

clone_trait_object!(EntityAttributeValueStorage);

#[derive(Clone, Debug)]
pub struct ExampleEntityAttributeValueStorageNonSync {
    storage: HashMap<HashString,EntityAttributeValue>,
}

impl ExampleEntityAttributeValueStorageNonSync {
    pub fn new() -> ExampleEntityAttributeValueStorageNonSync {
        ExampleEntityAttributeValueStorageNonSync {
            storage: HashMap::new(),
        }
    }

    fn unthreadable_add_eav(&mut self, eav: &EntityAttributeValue) -> Result<(), HolochainError> {
        let hash = HashString::from("");
        let key = vec![hash.to_string(),eav.address().to_string()].join("_");
        self.storage.insert(HashString::from(key),eav.clone());
        Ok(())
    }

    fn unthreadable_fetch_eav(
        &self,
        entity: Option<Entity>,
        attribute: Option<Attribute>,
        value: Option<Value>,
    ) -> Result<HashMap<HashString,EntityAttributeValue>, HolochainError> {
        let filtered = self.clone()
            .storage
            .into_iter()
          // .cloned()
            .filter(|(_,eav)| match entity {
                Some(ref e) => &eav.entity() == e,
                None => true,
            })
            .filter(|(_,eav)| match attribute {
                Some(ref a) => &eav.attribute() == a,
                None => true,
            })
            .filter(|(_,eav)| match value {
                Some(ref v) => &eav.value() == v,
                None => true,
            })
            .collect::<HashMap<HashString,EntityAttributeValue>>();
        Ok(filtered)
    }
}

impl PartialEq for EntityAttributeValueStorage {
    fn eq(&self, other: &EntityAttributeValueStorage) -> bool {
        self.fetch_eav(None, None, None) == other.fetch_eav(None, None, None)
    }
}

#[derive(Clone, Debug)]
pub struct ExampleEntityAttributeValueStorage {
    content: Arc<RwLock<ExampleEntityAttributeValueStorageNonSync>>,
}

impl ExampleEntityAttributeValueStorage {
    pub fn new() -> HcResult<ExampleEntityAttributeValueStorage> {
        Ok(ExampleEntityAttributeValueStorage {
            content: Arc::new(RwLock::new(ExampleEntityAttributeValueStorageNonSync::new())),
        })
    }
}

impl EntityAttributeValueStorage for ExampleEntityAttributeValueStorage {
    fn add_eav(&mut self, eav: &EntityAttributeValue) -> HcResult<()> {
        self.content.write().unwrap().unthreadable_add_eav(eav)
    }
    fn fetch_eav(
        &self,
        entity: Option<Entity>,
        attribute: Option<Attribute>,
        value: Option<Value>,
    ) -> Result<HashMap<HashString,EntityAttributeValue>, HolochainError> {
        self.content
            .read()
            .unwrap()
            .unthreadable_fetch_eav(entity, attribute, value)
    }

    fn get_hash(&self)->HashString
    {
        HashString::from("")
    }
}

pub fn test_eav_entity() -> Entry {
    test_entry_a()
}

pub fn test_eav_attribute() -> String {
    "foo-attribute".to_string()
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
    .expect("Could not create eav")
}

pub fn test_eav_content() -> Content {
    test_eav().content()
}

pub fn test_eav_address() -> Address {
    test_eav().address()
}

pub fn eav_round_trip_test_runner(
    entity_content: impl AddressableContent + Clone,
    attribute: String,
    value_content: impl AddressableContent + Clone,
) {
    let eav = EntityAttributeValue::new(
        &entity_content.address(),
        &attribute,
        &value_content.address(),
    )
    .expect("Could not create EAV");
    let mut eav_storage =
        ExampleEntityAttributeValueStorage::new().expect("could not create example eav storage");

    assert_eq!(
        HashMap::new(),
        eav_storage
            .fetch_eav(
                Some(entity_content.address()),
                Some(attribute.clone()),
                Some(value_content.address())
            )
            .expect("could not fetch eav"),
    );

    eav_storage.add_eav(&eav).expect("could not add eav");

    let mut expected = HashMap::new();
    expected.insert(HashString::from(""),eav.clone());
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

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::{
        cas::{
            content::{AddressableContent, AddressableContentTestSuite, ExampleAddressableContent},
            storage::{
                test_content_addressable_storage, EavTestSuite, ExampleContentAddressableStorage,
            },
        },
        eav::EntityAttributeValue,
        json::RawString,
    };

    pub fn test_eav_storage() -> ExampleEntityAttributeValueStorage {
        ExampleEntityAttributeValueStorage::new().expect("could not create example eav storage")
    }

    #[test]
    fn example_eav_round_trip() {
        let eav_storage = test_eav_storage();
        let entity =
            ExampleAddressableContent::try_from_content(&JsonString::from(RawString::from("foo")))
                .unwrap();
        let attribute = "favourite-color".to_string();
        let value =
            ExampleAddressableContent::try_from_content(&JsonString::from(RawString::from("blue")))
                .unwrap();

        EavTestSuite::test_round_trip(eav_storage, entity, attribute, value)
    }

    #[test]
    fn example_eav_one_to_many() {
        EavTestSuite::test_one_to_many::<
            ExampleAddressableContent,
            ExampleEntityAttributeValueStorage,
        >(test_eav_storage());
    }

    #[test]
    fn example_eav_many_to_one() {
        EavTestSuite::test_many_to_one::<
            ExampleAddressableContent,
            ExampleEntityAttributeValueStorage,
        >(test_eav_storage());
    }

    #[test]
    /// show AddressableContent implementation
    fn addressable_content_test() {
        // from_content()
        AddressableContentTestSuite::addressable_content_trait_test::<EntityAttributeValue>(
            test_eav_content(),
            test_eav(),
            test_eav_address(),
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

    #[test]
    fn validate_attribute_paths() {
        assert!(EntityAttributeValue::new(
            &test_eav_entity().address(),
            &"abc".to_string(),
            &test_eav_entity().address()
        )
        .is_ok());
        assert!(EntityAttributeValue::new(
            &test_eav_entity().address(),
            &"abc123".to_string(),
            &test_eav_entity().address()
        )
        .is_ok());
        assert!(EntityAttributeValue::new(
            &test_eav_entity().address(),
            &"123".to_string(),
            &test_eav_entity().address()
        )
        .is_ok());
        assert!(EntityAttributeValue::new(
            &test_eav_entity().address(),
            &"link_:{}".to_string(),
            &test_eav_entity().address()
        )
        .is_err());
        assert!(EntityAttributeValue::new(
            &test_eav_entity().address(),
            &"link_\"".to_string(),
            &test_eav_entity().address()
        )
        .is_err());
        assert!(EntityAttributeValue::new(
            &test_eav_entity().address(),
            &"link_/".to_string(),
            &test_eav_entity().address()
        )
        .is_err());
        assert!(EntityAttributeValue::new(
            &test_eav_entity().address(),
            &"link_\\".to_string(),
            &test_eav_entity().address()
        )
        .is_err());
        assert!(EntityAttributeValue::new(
            &test_eav_entity().address(),
            &"link_?".to_string(),
            &test_eav_entity().address()
        )
        .is_err());
    }

}
