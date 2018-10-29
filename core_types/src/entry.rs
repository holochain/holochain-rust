use cas::content::{Address, AddressableContent, Content};
use entry_type::{
    test_entry_type, test_entry_type_b, test_sys_entry_type, test_unpublishable_entry_type,
    EntryType,
};
use error::{error::HcResult, HolochainError};
use json::{JsonString, RawString, *};
use snowflake;
use std::{convert::TryFrom, ops::Deref};

pub type EntryValue = JsonString;

/// Structure holding actual data in a source chain "Item"
/// data is stored as a JsonString
#[derive(Clone, Debug)]
pub struct Entry {
    value: EntryValue,
    entry_type: EntryType,
}

impl Entry {
    pub fn new(entry_type: &EntryType, value: &JsonString) -> Entry {
        Entry {
            entry_type: entry_type.to_owned(),
            value: value.to_owned(),
        }
    }

    pub fn value(&self) -> &Content {
        &self.value
    }

    pub fn entry_type(&self) -> &EntryType {
        &self.entry_type
    }

    pub fn serialize(&self) -> SerializedEntry {
        SerializedEntry::from(self.clone())
    }
}

pub trait ToEntry {
    fn to_entry(&self) -> Entry;
    fn from_entry(&Entry) -> Self;
}

impl PartialEq for Entry {
    fn eq(&self, other: &Entry) -> bool {
        self.address() == other.address()
    }
}

/// entries are double serialized!
/// this struct facilitates the outer serialization
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SerializedEntry {
    value: String,
    entry_type: String,
}

impl SerializedEntry {
    pub fn new(entry_type: &str, value: &str) -> SerializedEntry {
        SerializedEntry {
            entry_type: entry_type.to_owned(),
            value: value.to_owned(),
        }
    }

    pub fn value(&self) -> String {
        self.value.clone()
    }

    pub fn entry_type(&self) -> String {
        self.entry_type.clone()
    }
}

// converting an Entry to SerializedEntry can never fail because it simply converts the fields
// to strings
impl From<Entry> for SerializedEntry {
    fn from(entry: Entry) -> SerializedEntry {
        SerializedEntry {
            value: String::from(entry.value()),
            entry_type: String::from(entry.entry_type().to_owned()),
        }
    }
}

impl From<SerializedEntry> for Entry {
    fn from(serialized_entry: SerializedEntry) -> Entry {
        Entry {
            value: JsonString::from(serialized_entry.value),
            entry_type: EntryType::from(serialized_entry.entry_type),
        }
    }
}

/// converting a SerializedEntry to JSON should never fail because it is a simple struct of strings
impl From<SerializedEntry> for JsonString {
    fn from(v: SerializedEntry) -> JsonString {
        default_to_json(v)
    }
}

/// restoring a JsonString to SerializedEntry can fail
impl TryFrom<JsonString> for SerializedEntry {
    type Error = HolochainError;
    fn try_from(json_string: JsonString) -> HcResult<Self> {
        default_try_from_json(json_string)
    }
}

// impl TryFrom<Option<SerializedEntry>> for JsonString {
//     type Error = HolochainError;
//     fn try_from(maybe_serialized_entry: Option<SerializedEntry>) -> JsonResult {
//         Ok(JsonString::from(format!(
//             "{{\"entry\":{}}}",
//             String::from(match maybe_serialized_entry {
//                 Some(serialized_entry) => JsonString::try_from(serialized_entry)?,
//                 None => JsonString::null(),
//             }),
//         )))
//     }
// }

impl AddressableContent for Entry {
    fn content(&self) -> Content {
        SerializedEntry::from(self.to_owned()).content()
    }

    fn from_content(content: &Content) -> Self {
        Self::from(
            SerializedEntry::try_from(content.to_owned()).expect("failed to restore Entry content"),
        )
    }
}

impl AddressableContent for SerializedEntry {
    fn content(&self) -> Content {
        Content::try_from(self.to_owned()).unwrap()
    }

    fn from_content(content: &Content) -> Self {
        Self::try_from(content.to_owned()).unwrap()
    }
}

impl Deref for Entry {
    type Target = Content;

    fn deref(&self) -> &Self::Target {
        self.value()
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
pub fn test_sys_entry_value() -> JsonString {
    // looks like a believable hash
    // sys entries are hashy right?
    JsonString::from(RawString::from(String::from(test_entry_value().address())))
}

/// dummy entry
#[cfg_attr(tarpaulin, skip)]
pub fn test_entry() -> Entry {
    Entry::new(&test_entry_type(), &test_entry_value())
}

pub fn test_serialized_entry() -> SerializedEntry {
    SerializedEntry {
        value: String::from(test_entry_value()),
        entry_type: String::from(test_entry_type()),
    }
}

pub fn expected_serialized_entry_content() -> JsonString {
    JsonString::from("{\"value\":\"\\\"test entry value\\\"\",\"entry_type\":\"testEntryType\"}")
}

/// the correct hash for test_entry()
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
    Entry::new(&test_entry_type_b(), &test_entry_value_b())
}
pub fn test_entry_c() -> Entry {
    Entry::new(&test_entry_type_b(), &test_entry_value_c())
}

/// dummy entry with unique string content
#[cfg_attr(tarpaulin, skip)]
pub fn test_entry_unique() -> Entry {
    Entry::new(
        &test_entry_type(),
        &JsonString::from(RawString::from(
            snowflake::ProcessUniqueId::new().to_string(),
        )),
    )
}

#[cfg_attr(tarpaulin, skip)]
pub fn test_sys_entry() -> Entry {
    Entry::new(&test_sys_entry_type(), &test_sys_entry_value())
}

pub fn test_sys_entry_address() -> Address {
    Address::from(String::from(
        "QmUZ3wsC4sVdJZK2AC8Ji4HZRfkFSH2cYE6FntmfWKF8GV",
    ))
}

#[cfg_attr(tarpaulin, skip)]
pub fn test_unpublishable_entry() -> Entry {
    Entry::new(&test_unpublishable_entry_type(), &test_entry().value())
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
    /// show From<Entry> for SerializedEntry
    fn serialized_entry_from_entry_test() {
        assert_eq!(test_serialized_entry(), SerializedEntry::from(test_entry()));
    }

    #[test]
    /// show From<SerializedEntry> for JsonString
    fn json_string_from_entry_test() {
        assert_eq!(
            test_entry().content(),
            JsonString::from(SerializedEntry::from(test_entry()))
        );
    }

    #[test]
    /// show From<SerializedEntry> for Entry
    fn entry_from_string_test() {
        assert_eq!(
            test_entry(),
            Entry::from(SerializedEntry::try_from(test_serialized_entry().content()).unwrap())
        );
    }

    #[test]
    /// tests for entry.content()
    fn content_test() {
        let content = test_entry_content();
        let entry = Entry::from_content(&content);

        assert_eq!(content, entry.content());
    }

    #[test]
    /// test that we can round trip through JSON
    fn json_round_trip() {
        let entry = test_entry();
        let expected = expected_serialized_entry_content();
        assert_eq!(
            expected,
            JsonString::from(SerializedEntry::from(entry.clone()))
        );
        assert_eq!(
            entry,
            Entry::from(SerializedEntry::try_from(expected.clone()).unwrap())
        );
        assert_eq!(entry, Entry::from(SerializedEntry::from(entry.clone())));

        let sys_entry = test_sys_entry();
        let expected = JsonString::from(format!(
            "{{\"value\":\"\\\"{}\\\"\",\"entry_type\":\"%agent_id\"}}",
            String::from(test_sys_entry_address()),
        ));
        assert_eq!(
            expected,
            JsonString::from(SerializedEntry::from(sys_entry.clone()))
        );
        assert_eq!(
            &sys_entry,
            &Entry::from(SerializedEntry::try_from(expected.clone()).unwrap())
        );
        assert_eq!(
            &sys_entry,
            &Entry::from(SerializedEntry::from(sys_entry.clone())),
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
