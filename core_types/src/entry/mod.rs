pub mod entry_type;
pub mod serde;

use cas::content::{Address, AddressableContent, Content};
use entry::entry_type::{
    EntryType,
};
use error::{HolochainError};
use json::{JsonString, RawString};
use snowflake;
use std::{
    convert::{TryFrom},
};
use agent::test_agent_id;
use dna::Dna;
use agent::AgentId;
use delete::Delete;
use link::link_add::LinkAdd;
use link::link_remove::LinkRemove;
use link::link_list::LinkList;
use chain_header::ChainHeader;
use chain_migrate::ChainMigrate;
use error::HcResult;
use entry::entry_type::SystemEntryType;
use entry::entry_type::AppEntryType;
use entry::entry_type::test_app_entry_type;
use entry::entry_type::test_app_entry_type_b;
use json::default_to_json;
use serde::Serializer;
use serde::Deserializer;
use serde::Deserialize;
use serde::ser::SerializeTupleVariant;

pub type AppEntryValue = JsonString;

fn serialize_app_entry <S>(app_entry_type: &AppEntryType, app_entry_value: &AppEntryValue, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
    let mut state = serializer.serialize_tuple_variant("Entry", 0, "App", 2)?;
    state.serialize_field(&app_entry_type.to_string())?;
    state.serialize_field(&app_entry_value.to_string())?;
    state.end()
}

fn deserialize_app_entry<'de, D>(deserializer: D) -> Result<(AppEntryType, AppEntryValue), D::Error> where D: Deserializer<'de> {
    #[derive(Deserialize)]
    struct SerializedAppEntry(String, String);

    let serialized_app_entry = SerializedAppEntry::deserialize(deserializer)?;
    Ok((AppEntryType::from(serialized_app_entry.0), AppEntryValue::from(serialized_app_entry.1)))
}

/// Structure holding actual data in a source chain "Item"
/// data is stored as a JsonString
#[derive(Clone, Debug, Serialize, Deserialize, DefaultJson)]
pub enum Entry {
    // @TODO don't skip
    #[serde(serialize_with="serialize_app_entry")]
    #[serde(deserialize_with="deserialize_app_entry")]
    App(AppEntryType, AppEntryValue),

    Dna(Dna),
    AgentId(AgentId),
    Delete(Delete),
    LinkAdd(LinkAdd),
    LinkRemove(LinkRemove),
    LinkList(LinkList),
    ChainHeader(ChainHeader),
    ChainMigrate(ChainMigrate),
}

impl From<Option<Entry>> for JsonString {
    fn from(maybe_entry: Option<Entry>) -> Self {
        default_to_json(maybe_entry)
    }
}

impl Entry {
    pub fn entry_type(&self) -> EntryType {
        match &self {
            Entry::App(app_entry_type, _) => EntryType::App(app_entry_type.to_owned()),
            Entry::Dna(_) => EntryType::System(SystemEntryType::Dna),
            Entry::AgentId(_) => EntryType::System(SystemEntryType::AgentId),
            Entry::Delete(_) => EntryType::System(SystemEntryType::Delete),
            Entry::LinkAdd(_) => EntryType::System(SystemEntryType::LinkAdd),
            Entry::LinkRemove(_) => EntryType::System(SystemEntryType::LinkRemove),
            Entry::LinkList(_) => EntryType::System(SystemEntryType::LinkList),
            Entry::ChainHeader(_) => EntryType::System(SystemEntryType::ChainHeader),
            Entry::ChainMigrate(_) => EntryType::System(SystemEntryType::ChainMigrate),
        }
    }
}

impl PartialEq for Entry {
    fn eq(&self, other: &Entry) -> bool {
        self.address() == other.address()
    }
}

impl AddressableContent for Entry {
    fn content(&self) -> Content {
        self.into()
    }

    fn try_from_content(content: &Content) -> HcResult<Entry> {
        Entry::try_from(content.to_owned())
    }
}

/// dummy entry value
#[cfg_attr(tarpaulin, skip)]
pub fn test_entry_value() -> JsonString {
    JsonString::from(RawString::from("test entry value"))
}

pub fn test_entry_content() -> Content {
    Content::from(r#"{"value":"\"test entry value\"","entry_type":"testEntryType"}"#)
}

/// dummy entry content, same as test_entry_value()
#[cfg_attr(tarpaulin, skip)]
pub fn test_entry_value_a() -> JsonString {
    test_entry_value()
}

/// dummy entry content, differs from test_entry_value()
#[cfg_attr(tarpaulin, skip)]
pub fn test_entry_value_b() -> JsonString {
    JsonString::from(RawString::from("other test entry value"))
}
#[cfg_attr(tarpaulin, skip)]
pub fn test_entry_value_c() -> JsonString {
    RawString::from("value C").into()
}

#[cfg_attr(tarpaulin, skip)]
pub fn test_sys_entry_value() -> AgentId {
    test_agent_id()
}

/// dummy entry
#[cfg_attr(tarpaulin, skip)]
pub fn test_entry() -> Entry {
    Entry::App(test_app_entry_type(), test_entry_value())
}

pub fn expected_serialized_entry_content() -> JsonString {
    JsonString::from("{\"value\":\"\\\"test entry value\\\"\",\"entry_type\":\"testEntryType\"}")
}

/// the correct address for test_entry()
#[cfg_attr(tarpaulin, skip)]
pub fn expected_entry_address() -> Address {
    Address::from("QmeoLRiWhXLTQKEAHxd8s6Yt3KktYULatGoMsaXi62e5zT".to_string())
}

/// dummy entry, same as test_entry()
#[cfg_attr(tarpaulin, skip)]
pub fn test_entry_a() -> Entry {
    test_entry()
}

/// dummy entry, differs from test_entry()
#[cfg_attr(tarpaulin, skip)]
pub fn test_entry_b() -> Entry {
    Entry::App(test_app_entry_type_b(), test_entry_value_b())
}
pub fn test_entry_c() -> Entry {
    Entry::App(test_app_entry_type_b(), test_entry_value_c())
}

/// dummy entry with unique string content
#[cfg_attr(tarpaulin, skip)]
pub fn test_entry_unique() -> Entry {
    Entry::App(
        test_app_entry_type(),
        RawString::from(snowflake::ProcessUniqueId::new().to_string()).into(),
    )
}

#[cfg_attr(tarpaulin, skip)]
pub fn test_sys_entry() -> Entry {
    Entry::AgentId(test_sys_entry_value())
}

pub fn test_sys_entry_address() -> Address {
    Address::from(String::from(
        "QmUZ3wsC4sVdJZK2AC8Ji4HZRfkFSH2cYE6FntmfWKF8GV",
    ))
}

#[cfg_attr(tarpaulin, skip)]
pub fn test_unpublishable_entry() -> Entry {
    Entry::Dna(Dna::new())
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use cas::{
        content::{AddressableContent, AddressableContentTestSuite},
        storage::{test_content_addressable_storage, ExampleContentAddressableStorage},
    };
    use entry::{expected_entry_address, Entry};

    #[test]
    /// tests for PartialEq
    fn eq() {
        let entry_a = test_entry_a();
        let entry_b = test_entry_b();

        // same content is equal
        assert_eq!(entry_a, entry_a);

        // different content is not equal
        assert_ne!(entry_a, entry_b);
    }

    #[test]
    /// test entry.address() against a known value
    fn known_address() {
        assert_eq!(expected_entry_address(), test_entry().address());
    }

    #[test]
    /// show From<Entry> for JsonString
    fn json_string_from_entry_test() {
        assert_eq!(
            test_entry().content(),
            JsonString::from(Entry::from(test_entry()))
        );
    }

    #[test]
    /// show From<Content> for Entry
    fn entry_from_content_test() {
        assert_eq!(
            test_entry(),
            Entry::try_from(test_entry().content()).unwrap()
        );
    }

    #[test]
    /// tests for entry.content()
    fn content_test() {
        let content = test_entry_content();
        let entry = Entry::try_from_content(&content).unwrap();

        assert_eq!(content, entry.content());
    }

    #[test]
    /// test that we can round trip through JSON
    fn json_round_trip() {
        let entry = test_entry();
        let expected = expected_serialized_entry_content();
        assert_eq!(
            expected,
            JsonString::from(Entry::from(entry.clone()))
        );
        assert_eq!(
            entry,
            Entry::from(Entry::try_from(expected.clone()).unwrap())
        );
        assert_eq!(entry, Entry::from(Entry::from(entry.clone())));

        let sys_entry = test_sys_entry();
        let expected = JsonString::from(format!(
            "{{\"value\":\"\\\"{}\\\"\",\"entry_type\":\"%agent_id\"}}",
            String::from(test_sys_entry_address()),
        ));
        assert_eq!(
            expected,
            JsonString::from(Entry::from(sys_entry.clone()))
        );
        assert_eq!(
            &sys_entry,
            &Entry::from(Entry::try_from(expected.clone()).unwrap())
        );
        assert_eq!(
            &sys_entry,
            &Entry::from(Entry::from(sys_entry.clone())),
        );
    }

    #[test]
    /// show AddressableContent implementation
    fn addressable_content_test() {
        // from_content()
        AddressableContentTestSuite::addressable_content_trait_test::<Entry>(
            test_entry_content(),
            test_entry(),
            expected_entry_address(),
        );
    }

    #[test]
    /// show CAS round trip
    fn cas_round_trip_test() {
        let entries = vec![test_entry()];
        AddressableContentTestSuite::addressable_content_round_trip::<
            Entry,
            ExampleContentAddressableStorage,
        >(entries, test_content_addressable_storage());
    }
}
