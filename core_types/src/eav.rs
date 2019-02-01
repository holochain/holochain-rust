//! EAV stands for entity-attribute-value. It is a pattern implemented here
//! for adding metadata about entries in the DHT, additionally
//! being used to define relationships between AddressableContent values.
//! See [wikipedia](https://en.wikipedia.org/wiki/Entity%E2%80%93attribute%E2%80%93value_model) to learn more about this pattern.

use crate::{
    cas::content::{Address, AddressableContent, Content},
    entry::{test_entry_a, test_entry_b, Entry},
    error::{HcResult, HolochainError},
    json::JsonString,
};
use chrono::offset::Utc;
use objekt;
use std::{
    cmp::Ordering,
    collections::BTreeSet,
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

// @TODO do we need this?
// unique (local to the source) monotonically increasing number that can be used for crdt/ordering
// @see https://papers.radixdlt.com/tempo/#logical-clocks
pub type Index = i64;

// @TODO do we need this?
// source agent asserting the meta
// type Source ...
/// The basic struct for EntityAttributeValue triple, implemented as AddressableContent
/// including the necessary serialization inherited.
#[derive(PartialEq, Eq, Hash, Clone, Debug, Serialize, Deserialize, DefaultJson, Default)]
pub struct EntityAttributeValueIndex {
    entity: Entity,
    attribute: Attribute,
    value: Value,
    index: Index,
    // source: Source,
}

impl PartialOrd for EntityAttributeValueIndex {
    fn partial_cmp(&self, other: &EntityAttributeValueIndex) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for EntityAttributeValueIndex {
    fn cmp(&self, other: &EntityAttributeValueIndex) -> Ordering {
        self.index.cmp(&other.index())
    }
}

#[derive(Clone, Debug)]
pub struct IndexQuery<'a> {
    start: Option<i64>,
    end: Option<i64>,
    prefixes: Vec<&'a str>,
}

impl<'a> IndexQuery<'a> {
    pub fn start(&self) -> Option<i64> {
        self.start.clone()
    }

    pub fn end(&self) -> Option<i64> {
        self.end.clone()
    }

    pub fn prefixes(&self) -> Vec<&str> {
        self.prefixes.clone()
    }
    pub fn new(start: i64, end: i64) -> IndexQuery<'a> {
        IndexQuery {
            start: Some(start),
            end: Some(end),
            prefixes: Vec::new(),
        }
    }

    pub fn new_with_options(start: Option<i64>, end: Option<i64>) -> IndexQuery<'a> {
        IndexQuery {
            start,
            end,
            prefixes: Vec::new(),
        }
    }

    pub fn new_only_prefixes(prefixes: Vec<&'a str>) -> IndexQuery<'a> {
        IndexQuery {
            start: None,
            end: None,
            prefixes,
        }
    }
}

impl<'a> Default for IndexQuery<'a> {
    fn default() -> IndexQuery<'a> {
        IndexQuery {
            start: None,
            end: None,
            prefixes: Vec::new(),
        }
    }
}

impl AddressableContent for EntityAttributeValueIndex {
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

impl EntityAttributeValueIndex {
    pub fn new(
        entity: &Entity,
        attribute: &Attribute,
        value: &Value,
    ) -> HcResult<EntityAttributeValueIndex> {
        validate_attribute(attribute)?;
        Ok(EntityAttributeValueIndex {
            entity: entity.clone(),
            attribute: attribute.clone(),
            value: value.clone(),
            index: Utc::now().timestamp_nanos(),
        })
    }

    pub fn new_with_index(
        entity: &Entity,
        attribute: &Attribute,
        value: &Value,
        timestamp: i64,
    ) -> HcResult<EntityAttributeValueIndex> {
        validate_attribute(attribute)?;
        Ok(EntityAttributeValueIndex {
            entity: entity.clone(),
            attribute: attribute.clone(),
            value: value.clone(),
            index: timestamp,
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

    pub fn index(&self) -> Index {
        self.index.clone()
    }

    pub fn set_index(&mut self, new_index: i64) {
        self.index = new_index
    }

    /// this is a predicate for matching on eav values. Useful for reducing duplicated filtered code.
    pub fn filter_on_eav<T>(eav: &T, e: Option<&T>) -> bool
    where
        T: PartialOrd,
    {
        e.map(|a| a == eav).unwrap_or(true)
    }

    /// this is a predicate for matching on eav values. Useful for reducing duplicated filtered code.
    pub fn filter_on_eav_with_prefix<'a>(
        eav: &'a String,
        e: Option<&'a String>,
        index_query: &'a IndexQuery<'a>,
    ) -> bool {
        let prefixes = if index_query.prefixes().len() > 0 {
            index_query.prefixes().clone()
        } else {
            vec![""]
        };
        e.map(|a| {
            prefixes.iter().any(|prefix| {
                let attribute_with_prefix = prefix.to_string() + &a.clone();
                attribute_with_prefix.clone() == eav.clone()
            })
        })
        .unwrap_or(true)
    }
}

/// This provides a simple and flexible interface to define relationships between AddressableContent.
/// It does NOT provide storage for AddressableContent.
/// Use cas::storage::ContentAddressableStorage to store AddressableContent.
pub trait EntityAttributeValueStorage: objekt::Clone + Send + Sync + Debug {
    /// Adds the given EntityAttributeValue to the EntityAttributeValueStorage
    /// append only storage.
    fn add_eavi(
        &mut self,
        eav: &EntityAttributeValueIndex,
    ) -> Result<Option<EntityAttributeValueIndex>, HolochainError>;
    /// Fetch the set of EntityAttributeValues that match constraints according to the latest hash version
    /// - None = no constraint
    /// - Some(Entity) = requires the given entity (e.g. all a/v pairs for the entity)
    /// - Some(Attribute) = requires the given attribute (e.g. all links)
    /// - Some(Value) = requires the given value (e.g. all entities referencing an Address)
    fn fetch_eavi(
        &self,
        entity: Option<Entity>,
        attribute: Option<Attribute>,
        value: Option<Value>,
        index_query: IndexQuery,
    ) -> Result<BTreeSet<EntityAttributeValueIndex>, HolochainError>;
}

clone_trait_object!(EntityAttributeValueStorage);

#[derive(Clone, Debug)]
pub struct ExampleEntityAttributeValueStorage {
    storage: Arc<RwLock<BTreeSet<EntityAttributeValueIndex>>>,
}
impl ExampleEntityAttributeValueStorage {
    pub fn new() -> ExampleEntityAttributeValueStorage {
        ExampleEntityAttributeValueStorage {
            storage: Arc::new(RwLock::new(BTreeSet::new())),
        }
    }
}

pub fn increment_key_till_no_collision(
    mut eav: EntityAttributeValueIndex,
    map: BTreeSet<EntityAttributeValueIndex>,
) -> HcResult<EntityAttributeValueIndex> {
    if map
        .iter()
        .filter(|e| e.index == eav.index())
        .collect::<BTreeSet<&EntityAttributeValueIndex>>()
        .len()
        > 0
    {
        let timestamp = eav.clone().index + 1;
        eav.set_index(timestamp);
        increment_key_till_no_collision(eav, map)
    } else {
        Ok(eav)
    }
}

impl EntityAttributeValueStorage for ExampleEntityAttributeValueStorage {
    fn add_eavi(
        &mut self,
        eav: &EntityAttributeValueIndex,
    ) -> Result<Option<EntityAttributeValueIndex>, HolochainError> {
        let mut map = self.storage.write()?;
        let new_eav = increment_key_till_no_collision(eav.clone(), map.clone())?;
        map.insert(new_eav.clone());
        Ok(Some(new_eav.clone()))
    }

    fn fetch_eavi(
        &self,
        entity: Option<Entity>,
        attribute: Option<Attribute>,
        value: Option<Value>,
        index_query: IndexQuery,
    ) -> Result<BTreeSet<EntityAttributeValueIndex>, HolochainError> {
        let map = self.storage.read()?;
        let new_map = map.clone();
        let filtered: BTreeSet<EntityAttributeValueIndex> = new_map
            .into_iter()
            .filter(|e| EntityAttributeValueIndex::filter_on_eav(&e.entity(), entity.as_ref()))
            .filter(|e| {
                EntityAttributeValueIndex::filter_on_eav_with_prefix(
                    &e.attribute(),
                    attribute.as_ref(),
                    &index_query,
                )
            })
            .filter(|e| EntityAttributeValueIndex::filter_on_eav(&e.value(), value.as_ref()))
            .collect();
        let filtered_dates = filtered
            .clone()
            .into_iter()
            .filter(|e| {
                index_query
                    .start()
                    .map(|start| start <= e.index())
                    .unwrap_or_else(|| {
                        let latest = filtered
                            .clone()
                            .into_iter()
                            .last()
                            .unwrap_or(EntityAttributeValueIndex::default());
                        latest.index() == e.index()
                    })
            })
            .filter(|e| {
                index_query
                    .end()
                    .map(|end| end >= e.index())
                    .unwrap_or_else(|| {
                        let latest = filtered
                            .clone()
                            .into_iter()
                            .last()
                            .unwrap_or(EntityAttributeValueIndex::default());
                        latest.index() == e.index()
                    })
            })
            .collect::<BTreeSet<EntityAttributeValueIndex>>();

        Ok(filtered_dates)
    }
}

pub fn get_latest(
    eav: EntityAttributeValueIndex,
    map: BTreeSet<EntityAttributeValueIndex>,
    index_query: IndexQuery,
) -> HcResult<EntityAttributeValueIndex> {
    let filter = map
        .clone()
        .into_iter()
        .filter(|e| EntityAttributeValueIndex::filter_on_eav(&e.entity(), Some(&eav.entity())))
        .filter(|e| {
            let prefixes = index_query.prefixes();
            let attribute_without_prefix =
                prefixes.iter().fold(eav.attribute(), |attribute, prefix| {
                    if eav.attribute().starts_with(prefix) {
                        let attri = attribute.clone();
                        attri.replace(prefix, "")
                    } else {
                        attribute.clone()
                    }
                });
            EntityAttributeValueIndex::filter_on_eav_with_prefix(
                &e.attribute(),
                Some(&attribute_without_prefix),
                &index_query,
            )
        })
        .filter(|e| EntityAttributeValueIndex::filter_on_eav(&e.value(), Some(&eav.value())));
    filter.last().ok_or(HolochainError::ErrorGeneric(
        "Could not get last value".to_string(),
    ))
}

impl PartialEq for EntityAttributeValueStorage {
    fn eq(&self, other: &EntityAttributeValueStorage) -> bool {
        self.fetch_eavi(None, None, None, IndexQuery::default())
            == other.fetch_eavi(None, None, None, IndexQuery::default())
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

pub fn test_eav() -> EntityAttributeValueIndex {
    EntityAttributeValueIndex::new_with_index(
        &test_eav_entity().address(),
        &test_eav_attribute(),
        &test_eav_value().address(),
        0,
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
    let eav = EntityAttributeValueIndex::new(
        &entity_content.address(),
        &attribute,
        &value_content.address(),
    )
    .expect("Could not create EAV");
    let mut eav_storage = ExampleEntityAttributeValueStorage::new();

    assert_eq!(
        BTreeSet::new(),
        eav_storage
            .fetch_eavi(
                Some(entity_content.address()),
                Some(attribute.clone()),
                Some(value_content.address()),
                IndexQuery::default()
            )
            .expect("could not fetch eav"),
    );

    eav_storage.add_eavi(&eav).expect("could not add eav");

    let mut expected = BTreeSet::new();
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
            eav_storage
                .fetch_eavi(e, a, v, IndexQuery::default())
                .expect("could not fetch eav")
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
        eav::EntityAttributeValueIndex,
        json::RawString,
    };

    pub fn test_eav_storage() -> ExampleEntityAttributeValueStorage {
        ExampleEntityAttributeValueStorage::new()
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
    fn example_eav_range() {
        EavTestSuite::test_range::<ExampleAddressableContent, ExampleEntityAttributeValueStorage>(
            test_eav_storage(),
        );
    }

    #[test]
    /// show AddressableContent implementation
    fn addressable_content_test() {
        // from_content()
        AddressableContentTestSuite::addressable_content_trait_test::<EntityAttributeValueIndex>(
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
            EntityAttributeValueIndex,
            ExampleContentAddressableStorage,
        >(addressable_contents, test_content_addressable_storage());
    }

    #[test]
    fn validate_attribute_paths() {
        assert!(EntityAttributeValueIndex::new(
            &test_eav_entity().address(),
            &"abc".to_string(),
            &test_eav_entity().address()
        )
        .is_ok());
        assert!(EntityAttributeValueIndex::new(
            &test_eav_entity().address(),
            &"abc123".to_string(),
            &test_eav_entity().address()
        )
        .is_ok());
        assert!(EntityAttributeValueIndex::new(
            &test_eav_entity().address(),
            &"123".to_string(),
            &test_eav_entity().address()
        )
        .is_ok());
        assert!(EntityAttributeValueIndex::new(
            &test_eav_entity().address(),
            &"link_:{}".to_string(),
            &test_eav_entity().address()
        )
        .is_err());
        assert!(EntityAttributeValueIndex::new(
            &test_eav_entity().address(),
            &"link_\"".to_string(),
            &test_eav_entity().address()
        )
        .is_err());
        assert!(EntityAttributeValueIndex::new(
            &test_eav_entity().address(),
            &"link_/".to_string(),
            &test_eav_entity().address()
        )
        .is_err());
        assert!(EntityAttributeValueIndex::new(
            &test_eav_entity().address(),
            &"link_\\".to_string(),
            &test_eav_entity().address()
        )
        .is_err());
        assert!(EntityAttributeValueIndex::new(
            &test_eav_entity().address(),
            &"link_?".to_string(),
            &test_eav_entity().address()
        )
        .is_err());
    }

}
