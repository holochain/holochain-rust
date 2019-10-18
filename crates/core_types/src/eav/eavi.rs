//! EAV stands for entity-attribute-value. It is a pattern implemented here
//! for adding metadata about entries in the DHT, additionally
//! being used to define relationships between AddressableContent values.
//! See [wikipedia](https://en.wikipedia.org/wiki/Entity%E2%80%93attribute%E2%80%93value_model) to learn more about this pattern.

use holochain_persistence_api::{
    cas::content::{Address, AddressableContent, Content},
    eav::{
        storage::{
            EntityAttributeValueStorage as GenericStorage, ExampleEntityAttributeValueStorage,
        },
        AttributeError, IndexFilter,
    },
    error::{PersistenceError, PersistenceResult},
};

use holochain_json_api::{error::JsonError, json::JsonString};

use crate::{
    entry::{test_entry_a, test_entry_b, Entry},
    error::HolochainError,
};

use regex::{Regex, RegexBuilder};
use std::{
    collections::BTreeSet,
    convert::{TryFrom, TryInto},
    fmt,
};

/// Address of AddressableContent representing the EAV entity
pub type Entity = Address;

/// All Attribute values are pre-defined here. If ever a user-defined Attribute is needed,
/// just add a new Custom variant for it with a String parameter
#[derive(PartialEq, Eq, PartialOrd, Hash, Clone, Debug, Serialize, Deserialize, DefaultJson)]
#[serde(rename_all = "snake_case")]
pub enum Attribute {
    CrudStatus,
    CrudLink,
    EntryHeader,
    Link,
    LinkRemove,
    LinkTag(String, String),
    RemovedLink(String, String),
    PendingEntry,
    Target,
}

impl Default for Attribute {
    fn default() -> Self {
        Attribute::EntryHeader
    }
}

unsafe impl Sync for Attribute {}
unsafe impl Send for Attribute {}

impl holochain_persistence_api::eav::Attribute for Attribute {}

impl From<AttributeError> for HolochainError {
    fn from(err: AttributeError) -> HolochainError {
        let msg = match err {
            AttributeError::Unrecognized(a) => format!("Unknown attribute: {}", a),
            AttributeError::ParseError => {
                String::from("Could not parse attribute, bad regex match")
            }
        };
        HolochainError::ErrorGeneric(msg)
    }
}

impl fmt::Display for Attribute {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Attribute::CrudStatus => write!(f, "crud-status"),
            Attribute::CrudLink => write!(f, "crud-link"),
            Attribute::EntryHeader => write!(f, "entry-header"),
            Attribute::Link => write!(f, "link"),
            Attribute::LinkRemove => write!(f, "link_remove"),
            Attribute::LinkTag(link_type, tag) => write!(f, "link__{}__{}", link_type, tag),
            Attribute::RemovedLink(link_type, tag) => {
                write!(f, "removed_link__{}__{}", link_type, tag)
            }
            Attribute::PendingEntry => write!(f, "pending-entry"),
            Attribute::Target => write!(f, "target"),
        }
    }
}

lazy_static! {
    static ref LINK_REGEX: Regex =
        Regex::new(r"^link__(.*)__(.*)$").expect("This string literal is a valid regex");
    static ref REMOVED_LINK_REGEX: Regex =
        Regex::new(r"^removed_link__(.*)__(.*)$").expect("This string literal is a valid regex");
}

impl TryFrom<&str> for Attribute {
    type Error = AttributeError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        use self::Attribute::*;
        if LINK_REGEX.is_match(s) {
            let link_type = LINK_REGEX.captures(s)?.get(1)?.as_str().to_string();
            let link_tag = LINK_REGEX.captures(s)?.get(2)?.as_str().to_string();

            Ok(LinkTag(link_type, link_tag))
        } else if REMOVED_LINK_REGEX.is_match(s) {
            let link_type = REMOVED_LINK_REGEX.captures(s)?.get(1)?.as_str().to_string();
            let link_tag = REMOVED_LINK_REGEX.captures(s)?.get(2)?.as_str().to_string();
            Ok(RemovedLink(link_type, link_tag))
        } else {
            match s {
                "crud-status" => Ok(CrudStatus),
                "crud-link" => Ok(CrudLink),
                "entry-header" => Ok(EntryHeader),
                "link" => Ok(Link),
                "link_remove" => Ok(LinkRemove),
                "pending-entry" => Ok(PendingEntry),
                "target" => Ok(Target),
                a => Err(AttributeError::Unrecognized(a.to_string())),
            }
        }
    }
}

impl TryFrom<String> for Attribute {
    type Error = AttributeError;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.as_str().try_into()
    }
}

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
pub type EntityAttributeValueIndex =
    holochain_persistence_api::eav::EntityAttributeValueIndex<Attribute>;

pub type EntityAttributeValueStorage = dyn GenericStorage<Attribute>;

fn validate_attribute(attribute: &Attribute) -> PersistenceResult<()> {
    if let Attribute::LinkTag(name, _tag) | Attribute::RemovedLink(name, _tag) = attribute {
        let regex = RegexBuilder::new(r#"[/:*?<>"'\\|+]"#)
            .build()
            .map_err(|_| PersistenceError::ErrorGeneric("Could not create regex".to_string()))?;
        if !regex.is_match(name) {
            Ok(())
        } else {
            Err(PersistenceError::ErrorGeneric(
                "Attribute name invalid".to_string(),
            ))
        }
    } else {
        Ok(())
    }
}

pub fn new(
    entity: &Address,
    attr: &Attribute,
    value: &Value,
) -> PersistenceResult<EntityAttributeValueIndex> {
    validate_attribute(attr).and_then(|_| EntityAttributeValueIndex::new(entity, attr, value))
}

pub fn test_eav_entity() -> Entry {
    test_entry_a()
}

pub fn test_eav_attribute() -> Attribute {
    Attribute::LinkTag("foo-attribute".into(), "foo-tag".into())
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
    attribute: Attribute,
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
            .fetch_eavi(&crate::eav::query::EaviQuery::new(
                Some(entity_content.address()).into(),
                Some(attribute.clone()).into(),
                Some(value_content.address()).into(),
                IndexFilter::LatestByAttribute,
                None
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
                .fetch_eavi(&crate::eav::query::EaviQuery::new(
                    e.into(),
                    a.into(),
                    v.into(),
                    IndexFilter::LatestByAttribute,
                    None
                ))
                .expect("could not fetch eav")
        );
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    use holochain_json_api::json::RawString;

    use holochain_persistence_api::cas::{
        content::{AddressableContent, AddressableContentTestSuite, ExampleAddressableContent},
        storage::{
            test_content_addressable_storage, EavTestSuite, ExampleContentAddressableStorage,
        },
    };

    use crate::eav::EntityAttributeValueIndex;

    pub fn test_eav_storage() -> ExampleEntityAttributeValueStorage<Attribute> {
        ExampleEntityAttributeValueStorage::new()
    }

    #[test]
    fn example_eav_round_trip() {
        let eav_storage = test_eav_storage();
        let entity =
            ExampleAddressableContent::try_from_content(&JsonString::from(RawString::from("foo")))
                .unwrap();
        let attribute = Attribute::LinkTag("abc".to_string(), "favourite-color".to_string());
        let value =
            ExampleAddressableContent::try_from_content(&JsonString::from(RawString::from("blue")))
                .unwrap();

        EavTestSuite::test_round_trip(eav_storage, entity, attribute, value)
    }

    #[test]
    fn example_eav_one_to_many() {
        EavTestSuite::test_one_to_many::<
            ExampleAddressableContent,
            Attribute,
            ExampleEntityAttributeValueStorage<Attribute>,
        >(test_eav_storage(), &Attribute::default());
    }

    #[test]
    fn example_eav_many_to_one() {
        EavTestSuite::test_many_to_one::<
            ExampleAddressableContent,
            Attribute,
            ExampleEntityAttributeValueStorage<Attribute>,
        >(test_eav_storage(), &Attribute::default());
    }

    #[test]
    fn example_eav_range() {
        EavTestSuite::test_range::<
            ExampleAddressableContent,
            Attribute,
            ExampleEntityAttributeValueStorage<Attribute>,
        >(test_eav_storage(), &Attribute::default());
    }

    #[test]
    fn example_eav_prefixes() {
        EavTestSuite::test_multiple_attributes::<
            ExampleAddressableContent,
            Attribute,
            ExampleEntityAttributeValueStorage<Attribute>,
        >(
            test_eav_storage(),
            vec!["a_", "b_", "c_", "d_"]
                .into_iter()
                .map(|p| Attribute::LinkTag(p.to_string() + "one_to_many", "".into()))
                .collect(),
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
    fn attribute_try_from_string() {
        assert_eq!("crud-status".try_into(), Ok(Attribute::CrudStatus));
        assert_eq!(
            "link__sometype__tagalog".try_into(),
            Ok(Attribute::LinkTag("sometype".into(), "tagalog".into()))
        );
        assert!(
            (r"unknown \\and// invalid / attribute".try_into() as Result<Attribute, _>).is_err(),
        );
    }

    #[test]
    fn validate_attribute_paths() {
        assert!(new(
            &test_eav_entity().address(),
            &Attribute::LinkTag("abc".into(), "".into()),
            &test_eav_entity().address()
        )
        .is_ok());
        assert!(new(
            &test_eav_entity().address(),
            &Attribute::LinkTag("abc123".into(), "".into()),
            &test_eav_entity().address()
        )
        .is_ok());
        assert!(new(
            &test_eav_entity().address(),
            &Attribute::LinkTag("123".into(), "".into()),
            &test_eav_entity().address()
        )
        .is_ok());
        assert!(new(
            &test_eav_entity().address(),
            &Attribute::LinkTag("link_:{}".into(), "".into()),
            &test_eav_entity().address()
        )
        .is_err());
        assert!(new(
            &test_eav_entity().address(),
            &Attribute::LinkTag("link_\"".into(), "".into()),
            &test_eav_entity().address()
        )
        .is_err());
        assert!(new(
            &test_eav_entity().address(),
            &Attribute::LinkTag("link_/".into(), "".into()),
            &test_eav_entity().address()
        )
        .is_err());
        assert!(new(
            &test_eav_entity().address(),
            &Attribute::LinkTag("link_\\".into(), "".into()),
            &test_eav_entity().address()
        )
        .is_err());
        assert!(new(
            &test_eav_entity().address(),
            &Attribute::LinkTag("link_?".into(), "".into()),
            &test_eav_entity().address()
        )
        .is_err());
    }

}
