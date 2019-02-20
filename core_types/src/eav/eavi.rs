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
use eav::{
    query::{EaviQuery, IndexFilter},
    storage::{EntityAttributeValueStorage, ExampleEntityAttributeValueStorage},
};
use std::{cmp::Ordering, collections::BTreeSet, convert::TryInto};

use regex::RegexBuilder;

/// Address of AddressableContent representing the EAV entity
pub type Entity = Address;

/// Using String for EAV attributes (not e.g. an enum) keeps it simple and open
pub type Attribute = String; // BUT we totally could make it an enum with a Custom variant!

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
            .fetch_eavi(&EaviQuery::new(
                Some(entity_content.address()).into(),
                Some(attribute.clone()).into(),
                Some(value_content.address()).into(),
                IndexFilter::LatestByAttribute
            ))
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
                .fetch_eavi(&EaviQuery::new(
                    e.into(),
                    a.into(),
                    v.into(),
                    IndexFilter::LatestByAttribute
                ))
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
    fn example_eav_prefixes() {
        EavTestSuite::test_prefixes::<ExampleAddressableContent, ExampleEntityAttributeValueStorage>(
            test_eav_storage(),
            vec!["a_", "b_", "c_", "d_"],
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
