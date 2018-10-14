use cas::content::{Address, AddressableContent, Content};
use entry::entry_type::{
    test_entry_type, test_entry_type_b, test_sys_entry_type, test_unpublishable_entry_type,
    EntryType,
};
use error::HolochainError;
use json::{FromJson, ToJson};
use serde_json;
use snowflake;
use std::ops::Deref;

pub mod agent;
pub mod app;
pub mod chain_header;
pub mod chain_migrate;
pub mod delete;
pub mod dna;
pub mod entry_type;
pub mod link_add;
pub mod link_remove;

pub trait Entry: AddressableContent {
    fn entry_type(&self) -> &EntryType;
}

impl PartialEq for Entry {
    fn eq(&self, other: &Entry) -> bool {
        self.address() == other.address()
    }
}

impl From<String> for Entry {
    fn from(string: String) -> Self {
        Entry::from_content(&string)
    }
}

impl From<Entry> for String {
    fn from(entry: Entry) -> Self {
        entry.content()
    }
}

/// dummy entry value
#[cfg_attr(tarpaulin, skip)]
pub fn test_entry_value() -> String {
    "test entry value".into()
}

pub fn test_entry_content() -> Content {
    Content::from("{\"value\":\"test entry value\",\"entry_type\":{\"App\":\"testEntryType\"}}")
}

/// dummy entry content, same as test_entry_content()
#[cfg_attr(tarpaulin, skip)]
pub fn test_entry_value_a() -> String {
    test_entry_value()
}

/// dummy entry content, differs from test_entry_content()
#[cfg_attr(tarpaulin, skip)]
pub fn test_entry_value_b() -> String {
    "other test entry value".into()
}

#[cfg_attr(tarpaulin, skip)]
pub fn test_sys_entry_value() -> String {
    // looks like a believable hash
    // sys entries are hashy right?
    test_entry_value().address().into()
}

/// dummy entry
#[cfg_attr(tarpaulin, skip)]
pub fn test_entry() -> Entry {
    Entry::new(&test_entry_type(), &test_entry_value())
}

/// the correct hash for test_entry()
#[cfg_attr(tarpaulin, skip)]
pub fn test_entry_address() -> Address {
    Address::from("QmW6oc9WdGJFf2C789biPLKbRWS1XD2sHrH5kYZVKqSwSr".to_string())
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

/// dummy entry with unique string content
#[cfg_attr(tarpaulin, skip)]
pub fn test_entry_unique() -> Entry {
    Entry::new(
        &test_entry_type(),
        &snowflake::ProcessUniqueId::new().to_string(),
    )
}

#[cfg_attr(tarpaulin, skip)]
pub fn test_sys_entry() -> Entry {
    Entry::new(&test_sys_entry_type(), &test_sys_entry_value())
}

pub fn test_sys_entry_address() -> Address {
    Address::from("QmWePdZYQrYFBUkBy1GPyCCUf8UmkmptsjtcVqZJ9Tzdse".to_string())
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
    use entry::{test_entry_address, Entry};
    use json::{FromJson, ToJson};

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
        assert_eq!(test_entry_address(), test_entry().address());
    }

    #[test]
    /// show From<Entry> for String
    fn string_from_entry_test() {
        assert_eq!(test_entry().content(), String::from(test_entry()));
    }

    #[test]
    /// show From<String> for Entry
    fn entry_from_string_test() {
        assert_eq!(test_entry(), Entry::from(test_entry().content()));
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
        let expected = test_entry_content();
        assert_eq!(expected, entry.to_json().unwrap());
        assert_eq!(entry, Entry::from_json(&expected).unwrap());
        assert_eq!(entry, Entry::from_json(&entry.to_json().unwrap()).unwrap());

        let sys_entry = test_sys_entry();
        let expected = format!(
            "{{\"value\":\"{}\",\"entry_type\":\"AgentId\"}}",
            test_sys_entry_address(),
        );
        assert_eq!(expected, sys_entry.to_json().unwrap());
        assert_eq!(sys_entry, Entry::from_json(&expected).unwrap());
        assert_eq!(
            sys_entry,
            Entry::from_json(&sys_entry.to_json().unwrap()).unwrap()
        );
    }

    #[test]
    /// show AddressableContent implementation
    fn addressable_content_test() {
        // from_content()
        AddressableContentTestSuite::addressable_content_trait_test::<Entry>(
            test_entry_content(),
            test_entry(),
            String::from(test_entry_address()),
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
