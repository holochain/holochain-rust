use crate::{
    cas::content::{Address, AddressableContent, Content},
    eav::EntityAttributeValue,
    error::error::HolochainError,
    hash::HashString,
    json::JsonString,
};
use std::convert::TryInto;

// @TODO are these the correct key names?
// @see https://github.com/holochain/holochain-rust/issues/143
pub const STATUS_NAME: &str = "crud-status";
pub const LINK_NAME: &str = "crud-link";

pub fn create_crud_status_eav(address: &Address, status: CrudStatus) -> EntityAttributeValue {
    EntityAttributeValue::new(
        address,
        &STATUS_NAME.to_string(),
        &HashString::from(String::from(status)),
    )
}

pub fn create_crud_link_eav(from: &Address, to: &Address) -> EntityAttributeValue {
    EntityAttributeValue::new(from, &LINK_NAME.to_string(), to)
}

/// the CRUD status of a Pair is stored as EntryMeta in the hash table, NOT in the entry itself
#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize, DefaultJson)]
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

impl From<String> for CrudStatus {
    fn from(s: String) -> CrudStatus {
        let str_gulp: String = format!("{:?}", s);

        serde_json::from_str(str_gulp.as_ref()).expect("failed to deserialize into CrudStatus enum")
    }
}

impl AddressableContent for CrudStatus {
    fn content(&self) -> Content {
        self.to_owned().into()
    }

    fn try_from_content(content: &Content) -> Result<Self, HolochainError> {
        content.to_owned().try_into()
    }
}

#[cfg(test)]
mod tests {
    use super::CrudStatus;
    use crate::{
        cas::{
            content::{
                Address, AddressableContent, AddressableContentTestSuite, Content,
                ExampleAddressableContent,
            },
            storage::{test_content_addressable_storage, ExampleContentAddressableStorage},
        },
        eav::eav_round_trip_test_runner,
        json::{JsonString, RawString},
    };

    #[test]
    fn crud_status_example_eav() {
        let entity_content = ExampleAddressableContent::try_from_content(&JsonString::from(
            RawString::from("example"),
        ))
        .unwrap();
        let attribute = String::from("favourite-badge");
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
