use cas::content::{Address, AddressableContent, Content};
use entry_type::EntryType;
use error::HolochainError;
use json::{FromJson, ToJson};
use serde_json;
use snowflake;
use std::ops::Deref;

/// Structure holding actual data in a source chain "Item"
/// data is stored as a JSON string
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entry(String);

impl PartialEq for Entry {
    fn eq(&self, other: &Entry) -> bool {
        self.address() == other.address()
    }
}

impl From<String> for Entry {
    fn from(string: String) -> Self {
        Entry(string)
    }
}

impl From<Entry> for String {
    fn from(entry: Entry) -> Self {
        entry.0
    }
}

impl AddressableContent for Entry {
    fn content(&self) -> Content {
        String::from(self.to_owned())
    }

    fn from_content(content: &Content) -> Self {
        Entry::from(content.to_string())
    }
}

impl Entry {
    pub fn new() -> Entry {
        Entry(String::new())
    }
}

impl ToJson for Entry {
    /// @TODO return canonical JSON
    /// @see https://github.com/holochain/holochain-rust/issues/75
    fn to_json(&self) -> Result<String, HolochainError> {
        Ok(serde_json::to_string(&self)?)
    }
}

impl FromJson for Entry {
    /// @TODO accept canonical JSON
    /// @see https://github.com/holochain/holochain-rust/issues/75
    fn from_json(s: &str) -> Result<Self, HolochainError> {
        Ok(serde_json::from_str(s)?)
    }
}

impl Deref for Entry {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// dummy entry type
pub fn test_entry_type() -> EntryType {
    EntryType::App(String::from("testEntryType"))
}

/// dummy entry type, same as test_type()
pub fn test_entry_type_a() -> EntryType {
    test_entry_type()
}

/// dummy entry type, differs from test_type()
pub fn test_entry_type_b() -> EntryType {
    EntryType::App(String::from("testEntryTypeB"))
}

/// dummy entry content
pub fn test_entry_content() -> String {
    "test entry content".into()
}

/// dummy entry content, same as test_entry_content()
pub fn test_entry_content_a() -> String {
    test_entry_content()
}

/// dummy entry content, differs from test_entry_content()
pub fn test_entry_content_b() -> String {
    "other test entry content".into()
}

pub fn test_sys_entry_content() -> String {
    // looks like a believable hash
    // sys entries are hashy right?
    test_entry_content().address().into()
}

/// dummy entry
pub fn test_entry() -> Entry {
    Entry::from_content(&test_entry_content())
}

/// the correct hash for test_entry()
pub fn test_entry_address() -> Address {
    Address::from("QmbXSE38SN3SuJDmHKSSw5qWWegvU7oTxrLDRavWjyxMrT".to_string())
}

/// dummy entry, same as test_entry()
pub fn test_entry_a() -> Entry {
    test_entry()
}

/// dummy entry, differs from test_entry()
pub fn test_entry_b() -> Entry {
    Entry::from_content(&test_entry_content_b())
}

/// dummy entry with unique string content
pub fn test_entry_unique() -> Entry {
    Entry::from_content(&snowflake::ProcessUniqueId::new().to_string())
}

pub fn test_sys_entry() -> Entry {
    Entry::from_content(&test_sys_entry_content())
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use cas::{
        content::{AddressableContent, AddressableContentTestSuite},
        storage::{test_content_addressable_storage, ExampleContentAddressableStorage},
    };
    use entry::Entry;
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
        assert_eq!(test_entry_content().to_string(), String::from(test_entry()));
    }

    #[test]
    /// show From<String> for Entry
    fn entry_from_string_test() {
        assert_eq!(test_entry(), Entry::from(test_entry_content().to_string()));
    }

    #[test]
    /// tests for entry.content()
    fn content() {
        let content = "baz";
        let entry = Entry::from_content(&String::from(content));

        assert_eq!("baz", entry.content());
    }

    #[test]
    /// test that we can round trip through JSON
    fn json_round_trip() {
        let entry = test_entry();
        let expected = r#""test entry content""#;
        assert_eq!(expected, entry.to_json().unwrap());
        assert_eq!(entry, Entry::from_json(expected).unwrap());
        assert_eq!(entry, Entry::from_json(&entry.to_json().unwrap()).unwrap());
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
