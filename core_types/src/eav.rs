//! EAV stands for entity-attribute-value. It is a pattern implemented here
//! for adding metadata about entries in the DHT, additionally
//! being used to define relationships between AddressableContent values.
//! See [wikipedia](https://en.wikipedia.org/wiki/Entity%E2%80%93attribute%E2%80%93value_model) to learn more about this pattern.

use crate::{
    cas::content::{Address, AddressableContent, Content},
    entry::{test_entry_a, test_entry_b, Entry},
    error::{HcResult, HolochainError},
    hash::HashString,
    json::JsonString,
};
use chrono::{offset::Utc, DateTime};
use im::ordmap::OrdMap;
use objekt;
use std::{
    convert::TryInto,
    sync::{Arc, RwLock},
};

use regex::RegexBuilder;
use std::fmt::Debug;

/// Address of AddressableContent representing the EAV entity
pub type Entity = Address;

/// Using String for EAV attributes (not e.g. an enum) keeps it simple and open
pub type Attribute = String;

/// Address of AddressableContent representing the EAV value
pub type Value = Address;

#[derive(PartialEq, Eq, Debug, Clone, Hash, PartialOrd, Ord)]
pub struct Key(pub i64, pub Action);

// @TODO do we need this?
// unique (local to the source) monotonically increasing number that can be used for crdt/ordering
// @see https://papers.radixdlt.com/tempo/#logical-clocks
// type Index ...

// @TODO do we need this?
// source agent asserting the meta
// type Source ...
/// The basic struct for EntityAttributeValue triple, implemented as AddressableContent
/// including the necessary serialization inherited.
#[derive(
    PartialEq, Eq, Hash, Clone, Debug, Serialize, Deserialize, DefaultJson, Default, PartialOrd, Ord,
)]
pub struct EntityAttributeValue {
    entity: Entity,
    attribute: Attribute,
    value: Value,
    // index: Index,
    // source: Source,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Action {
    Insert,
    Delete,
    Update,
    None,
}

impl ToString for Action {
    fn to_string(&self) -> String {
        match self {
            Action::Insert => String::from("Insert"),
            Action::Delete => String::from("Delete"),
            _ => String::from("None"),
        }
    }
}

impl From<String> for Action {
    fn from(action: String) -> Self {
        if action == String::from("Insert") {
            Action::Insert
        } else if action == String::from("Delete") {
            Action::Delete
        } else {
            Action::None
        }
    }
}

pub fn create_key(action: Action) -> Result<Key, HolochainError> {
    Ok(Key(Utc::now().timestamp_millis(), action))
}

pub fn from_key(key: HashString) -> Result<Key, HolochainError> {
    let string_key = key.to_string();
    let split = string_key.split("_").collect::<Vec<&str>>();
    let mut split_iter = split.iter();
    let timestamp = split_iter.next().ok_or(HolochainError::ErrorGeneric(
        "Could not get timestamp".to_string(),
    ))?;
    let action = split_iter.next().ok_or(HolochainError::ErrorGeneric(
        "Could not get action".to_string(),
    ))?;
    let unix_timestamp = timestamp
        .parse::<i64>()
        .map_err(|_| HolochainError::ErrorGeneric("Could not get action".to_string()))?;
    Ok(Key(
        unix_timestamp,
        Action::from(action.clone().to_string()),
    ))
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

/// This provides a simple and flexible interface to define relationships between AddressableContent.
/// It does NOT provide storage for AddressableContent.
/// Use cas::storage::ContentAddressableStorage to store AddressableContent.
pub trait EntityAttributeValueStorage: objekt::Clone + Send + Sync + Debug {
    /// Adds the given EntityAttributeValue to the EntityAttributeValueStorage
    /// append only storage.
    fn add_eav(&mut self, eav: &EntityAttributeValue) -> Result<(), HolochainError>;
    /// Fetch the set of EntityAttributeValues that match constraints according to the latest hash version
    /// - None = no constraint
    /// - Some(Entity) = requires the given entity (e.g. all a/v pairs for the entity)
    /// - Some(Attribute) = requires the given attribute (e.g. all links)
    /// - Some(Value) = requires the given value (e.g. all entities referencing an Address)
    fn fetch_eav(
        &self,
        entity: Option<Entity>,
        attribute: Option<Attribute>,
        value: Option<Value>,
    ) -> Result<OrdMap<Key, EntityAttributeValue>, HolochainError>;

    //optimize this according to the trait store
    fn fetch_eav_range(
        &self,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
        entity: Option<Entity>,
        attribute: Option<Attribute>,
        value: Option<Value>,
    ) -> Result<OrdMap<Key, EntityAttributeValue>, HolochainError> {
        let eavs = self.fetch_eav(entity, attribute, value)?;
        Ok(eavs
            .iter()
            .cloned()
            .filter(|(key, _)| {
                key.0
                    <= start_date
                        .map(|s| s.timestamp_millis())
                        .unwrap_or(i64::min_value())
                    && end_date
                        .map(|s| s.timestamp_millis())
                        .unwrap_or(i64::max_value())
                        >= key.0
            })
            .collect())
    }
}

clone_trait_object!(EntityAttributeValueStorage);

#[derive(Clone, Debug)]
pub struct ExampleEntityAttributeValueStorageNonSync {
    storage: OrdMap<Key, EntityAttributeValue>,
}

impl ExampleEntityAttributeValueStorageNonSync {
    pub fn new() -> ExampleEntityAttributeValueStorageNonSync {
        ExampleEntityAttributeValueStorageNonSync {
            storage: OrdMap::new(),
        }
    }

    fn unthreadable_add_eav(&mut self, eav: &EntityAttributeValue) -> Result<(), HolochainError> {
        if self
            .unthreadable_fetch_eav(Some(eav.entity()), Some(eav.attribute()), Some(eav.value()))?
            .len()
            == 0
        {
            let key = create_key(Action::Insert)?;
            self.storage.insert(key, eav.clone());
            Ok(())
        } else {
            Ok(())
        }
    }

    fn unthreadable_fetch_eav(
        &self,
        entity: Option<Entity>,
        attribute: Option<Attribute>,
        value: Option<Value>,
    ) -> Result<OrdMap<Key, EntityAttributeValue>, HolochainError> {
        let filtered = self
            .clone()
            .storage
            .into_iter()
            // .cloned()
            .filter(|(_, eav)| match entity {
                Some(ref e) => &eav.entity() == e,
                None => true,
            })
            .filter(|(_, eav)| match attribute {
                Some(ref a) => &eav.attribute() == a,
                None => true,
            })
            .filter(|(_, eav)| match value {
                Some(ref v) => &eav.value() == v,
                None => true,
            })
            .collect::<OrdMap<Key, EntityAttributeValue>>();
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
    ) -> Result<OrdMap<Key, EntityAttributeValue>, HolochainError> {
        self.content
            .read()
            .unwrap()
            .unthreadable_fetch_eav(entity, attribute, value)
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
        OrdMap::new(),
        eav_storage
            .fetch_eav(
                Some(entity_content.address()),
                Some(attribute.clone()),
                Some(value_content.address())
            )
            .expect("could not fetch eav"),
    );

    eav_storage.add_eav(&eav).expect("could not add eav");

    let mut expected = OrdMap::new();
    let key = create_key(Action::Insert).expect("Could not create key");
    expected.insert(key, eav.clone());
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
