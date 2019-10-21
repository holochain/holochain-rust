//! This module holds the relevant constants and an enum required for Holochain to have 'status' metadata for entries.
//! Since Holochain uses an append-only data structure, but still wishes to provide classical features of a data
//! store such as "update" and "remove" (delete), metadata is created pointing entries forward to their 'latest' version,
//! even including an entry being marked as deleted.

use crate::eav::EntityAttributeValueIndex;

use eav::Attribute;
use std::{convert::TryInto, str::FromStr};

use holochain_persistence_api::{
    cas::content::{Address, AddressableContent, Content},
    error::PersistenceResult,
    hash::HashString,
};

use holochain_json_api::{
    error::{JsonError, JsonResult},
    json::JsonString,
};

/// Create a new [EAV](../eav/struct.EntityAttributeValue.html) with an entry address as the Entity, [CrudStatus](../eav/Attribute.html) as the attribute
/// and CrudStatus as the value.
/// This will come to represent the lifecycle status of an entry, when it gets stored in an [EAV Storage](../eav/trait.EntityAttributeValueStorage.html)
pub fn create_crud_status_eav(
    address: &Address,
    status: CrudStatus,
) -> PersistenceResult<EntityAttributeValueIndex> {
    EntityAttributeValueIndex::new(
        address,
        &Attribute::CrudStatus,
        &HashString::from(String::from(status)),
    )
}

/// Create a new [EAV](../eav/struct.EntityAttributeValue.html) with an old entry address as the Entity, [CrudLink](../eav/Attribute.html) as the attribute
/// and a new entry address as the value
pub fn create_crud_link_eav(
    from: &Address,
    to: &Address,
) -> PersistenceResult<EntityAttributeValueIndex> {
    EntityAttributeValueIndex::new(from, &Attribute::CrudLink, to)
}

/// the CRUD status of a Pair is stored using an EAV, NOT in the entry itself
#[derive(
    Copy, Clone, PartialEq, Debug, Serialize, Deserialize, DefaultJson, PartialOrd, Ord, Eq,
)]
#[serde(rename_all = "lowercase")]
pub enum CrudStatus {
    Live,
    Rejected,
    Deleted,
    Modified,
    /// CRDT resolution in progress
    Locked,
}

impl Default for CrudStatus {
    fn default() -> CrudStatus {
        CrudStatus::Live
    }
}

impl From<CrudStatus> for String {
    fn from(status: CrudStatus) -> String {
        let res_str = serde_json::to_string(&status).expect("failed to serialize CrudStatus enum");

        res_str.chars().filter(|kar| kar.is_alphabetic()).collect()
    }
}

impl FromStr for CrudStatus {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let quoted_string = format!("{:?}", s);

        let res = serde_json::from_str(quoted_string.as_ref())
            .map_err(|_| "failed to deserialize CrudStatus enum")?;

        Ok(res)
    }
}

impl AddressableContent for CrudStatus {
    fn content(&self) -> Content {
        self.to_owned().into()
    }

    fn try_from_content(content: &Content) -> JsonResult<Self> {
        content.to_owned().try_into()
    }
}

#[cfg(test)]
mod tests {
    use super::CrudStatus;
    use crate::eav::{eav_round_trip_test_runner, Attribute};

    use holochain_json_api::json::{JsonString, RawString};
    use holochain_persistence_api::cas::{
        content::{
            Address, AddressableContent, AddressableContentTestSuite, Content,
            ExampleAddressableContent,
        },
        storage::{test_content_addressable_storage, ExampleContentAddressableStorage},
    };

    #[test]
    fn crud_status_example_eav() {
        let entity_content = ExampleAddressableContent::try_from_content(&JsonString::from(
            RawString::from("example"),
        ))
        .unwrap();
        let attribute = Attribute::LinkTag("favourite-badge".into(), "some-tag".into());
        let value_content: Content =
            CrudStatus::try_from_content(&JsonString::from(CrudStatus::Rejected))
                .unwrap()
                .content();
        eav_round_trip_test_runner(entity_content, attribute, value_content);
    }

    #[test]
    /// show AddressableContent implementation
    fn addressable_content_test() {
        // from_content()
        AddressableContentTestSuite::addressable_content_trait_test::<CrudStatus>(
            JsonString::from(CrudStatus::Live),
            CrudStatus::Live,
            Address::from("QmXEyo1EepSNCmZjPzGATr8BF3GMYAKKSXbWJN9QS95jLx"),
        );
        AddressableContentTestSuite::addressable_content_trait_test::<CrudStatus>(
            JsonString::from(CrudStatus::Rejected),
            CrudStatus::Rejected,
            Address::from("QmcifaUPPN6BBmpjakau1DGx9nFb9YhoS7fZjPHwFLoRuw"),
        );
        AddressableContentTestSuite::addressable_content_trait_test::<CrudStatus>(
            JsonString::from(CrudStatus::Deleted),
            CrudStatus::Deleted,
            Address::from("QmVKAvoNaU3jrKEvPK9tc6ovJWozxS9CVuNfWB4sbbYwR9"),
        );
        AddressableContentTestSuite::addressable_content_trait_test::<CrudStatus>(
            JsonString::from(CrudStatus::Modified),
            CrudStatus::Modified,
            Address::from("QmbJmMc19gp8jNAoHSk5qY5H6LUF9qp9LTSKWFZojToAEz"),
        );
        AddressableContentTestSuite::addressable_content_trait_test::<CrudStatus>(
            JsonString::from(CrudStatus::Locked),
            CrudStatus::Locked,
            Address::from("QmUjxgPiP7wxpowjWD9t7FLrGgnNmNA1FUGHVY3BrEnKe3"),
        );
    }

    #[test]
    /// show CAS round trip
    fn cas_round_trip_test() {
        let crud_statuses = vec![
            CrudStatus::Live,
            CrudStatus::Rejected,
            CrudStatus::Deleted,
            CrudStatus::Modified,
            CrudStatus::Locked,
        ];
        AddressableContentTestSuite::addressable_content_round_trip::<
            CrudStatus,
            ExampleContentAddressableStorage,
        >(crud_statuses, test_content_addressable_storage());
    }
}
