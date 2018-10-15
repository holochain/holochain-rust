use cas::content::{Address, AddressableContent, Content};
use entry::{
    agent::AgentId,
    chain_header::ChainHeader,
    chain_migrate::ChainMigrate,
    delete::Delete,
    dna::{test_dna, Dna},
    link_add::LinkAdd,
    link_remove::LinkRemove,
};
use keys::test_key;
use serde_json;
use snowflake;
use std::fmt::{Display, Formatter, Result};
use json::JsonString;

pub mod agent;
pub mod app;
pub mod chain_header;
pub mod chain_migrate;
pub mod delete;
pub mod dna;
// pub mod entry_type;
pub mod link_add;
pub mod link_remove;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Hash)]
pub struct AppEntryType(String);

impl Eq for AppEntryType {}

impl From<&'static str> for AppEntryType {
    fn from(s: &str) -> AppEntryType {
        AppEntryType(String::from(s))
    }
}

impl From<String> for AppEntryType {
    fn from(s: String) -> AppEntryType {
        AppEntryType(s)
    }
}

impl From<AppEntryType> for String {
    fn from(app_entry_type: AppEntryType) -> String {
        app_entry_type.0
    }
}

impl Display for AppEntryType {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppEntryValue(serde_json::Value);

impl From<String> for AppEntryValue {
    fn from(json: String) -> AppEntryValue {
        AppEntryValue(serde_json::from_str(&json).expect("could not deserialize test entry value"))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum EntryType {
    App(AppEntryType),
    Dna,
    AgentId,
    Delete,
    LinkAdd,
    LinkRemove,
    ChainHeader,
    ChainMigrate,
}

impl EntryType {
    pub fn can_publish(&self) -> bool {
        match self {
            EntryType::Dna => false,
            _ => true,
        }
    }

    pub fn is_sys(&self) -> bool {
        match self {
            EntryType::App(_) => false,
            _ => true,
        }
    }

    pub fn is_app(&self) -> bool {
        !self.is_sys()
    }
}

impl Display for EntryType {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.to_string())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Entry {
    Dna(Dna),

    AgentId(AgentId),

    App(AppEntryType, AppEntryValue),
    Delete(Delete),

    LinkAdd(LinkAdd),
    LinkRemove(LinkRemove),

    ChainHeader(ChainHeader),
    ChainMigrate(ChainMigrate),
}

impl Entry {
    pub fn entry_type(&self) -> EntryType {
        match self {
            Entry::Dna(_) => EntryType::Dna,
            Entry::AgentId(_) => EntryType::AgentId,
            Entry::App(app_entry_type, _) => EntryType::App(app_entry_type.to_owned()),
            Entry::Delete(_) => EntryType::Delete,
            Entry::LinkAdd(_) => EntryType::LinkAdd,
            Entry::LinkRemove(_) => EntryType::LinkRemove,
            Entry::ChainHeader(_) => EntryType::ChainHeader,
            Entry::ChainMigrate(_) => EntryType::ChainMigrate,
        }
    }
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

impl AddressableContent for Entry {
    fn content(&self) -> Content {
        serde_json::to_string(self).expect("could not Jsonify Entry as Content")
    }

    fn from_content(content: &Content) -> Entry {
        serde_json::from_str(content).expect("could not read Json as valid Entry")
    }
}

impl From<Entry> for JsonString {
    fn from(entry: Entry) -> JsonString {
        JsonString::from(entry.content())
    }
}

impl From<Option<Entry>> for JsonString {
    fn from (maybe_entry: Option<Entry>) -> JsonString {
        let inner = match maybe_entry {
            Some(entry) => JsonString::from(entry),
            None => JsonString::none(),
        };
        JsonString::from(format!("{{\"entry\": {}}}", inner))
    }
}

/// dummy entry type
#[cfg_attr(tarpaulin, skip)]
pub fn test_app_entry_type() -> AppEntryType {
    AppEntryType::from("testEntryType")
}

/// dummy entry type, same as test_type()
#[cfg_attr(tarpaulin, skip)]
pub fn test_app_entry_type_a() -> AppEntryType {
    test_app_entry_type()
}

/// dummy entry type, differs from test_type()
#[cfg_attr(tarpaulin, skip)]
pub fn test_app_entry_type_b() -> AppEntryType {
    AppEntryType::from("testEntryTypeB")
}

/// dummy entry value
#[cfg_attr(tarpaulin, skip)]
pub fn test_app_entry_value() -> AppEntryValue {
    #[derive(Serialize)]
    struct A {
        foo: String,
        bar: Vec<String>,
    }
    let a = A {
        foo: "test entry value".to_owned(),
        bar: vec!["bing".to_owned(), "baz".to_owned()],
    };
    let json = serde_json::to_string(&a).expect("could not serialize test entry value");
    AppEntryValue::from(json)
}

#[cfg_attr(tarpaulin, skip)]
pub fn expected_app_entry_content() -> Content {
    Content::from("{\"value\":\"test entry value\",\"entry_type\":{\"App\":\"testEntryType\"}}")
}

/// dummy entry content, same as test_entry_content()
#[cfg_attr(tarpaulin, skip)]
pub fn test_app_entry_value_a() -> AppEntryValue {
    test_app_entry_value()
}

/// dummy entry content, differs from test_entry_content()
#[cfg_attr(tarpaulin, skip)]
pub fn test_app_entry_value_b() -> AppEntryValue {
    #[derive(Serialize)]
    struct B {
        x: i32,
        y: i32,
    }
    let b = B { x: 10, y: 200 };
    let json = serde_json::to_string(&b).expect("could not serialize test entry value");
    AppEntryValue::from(json)
}

/// dummy entry
#[cfg_attr(tarpaulin, skip)]
pub fn test_app_entry() -> Entry {
    Entry::App(test_app_entry_type(), test_app_entry_value())
}

/// the correct hash for test_app_entry()
#[cfg_attr(tarpaulin, skip)]
pub fn expected_add_entry_address() -> Address {
    Address::from("QmW6oc9WdGJFf2C789biPLKbRWS1XD2sHrH5kYZVKqSwSr".to_string())
}

/// dummy entry, same as test_entry()
#[cfg_attr(tarpaulin, skip)]
pub fn test_app_entry_a() -> Entry {
    test_app_entry()
}

/// dummy entry, differs from test_entry()
#[cfg_attr(tarpaulin, skip)]
pub fn test_app_entry_b() -> Entry {
    Entry::App(test_app_entry_type_b(), test_app_entry_value_b())
}

pub fn test_app_entry_value_unique() -> AppEntryValue {
    #[derive(Serialize)]
    struct Unique {
        id: String,
    }
    let unique = Unique {
        id: snowflake::ProcessUniqueId::new().to_string(),
    };
    let json = serde_json::to_string(&unique).expect("could not serialize test entry value");
    AppEntryValue::from(json)
}

/// dummy entry with unique string content
#[cfg_attr(tarpaulin, skip)]
pub fn test_app_entry_unique() -> Entry {
    Entry::App(test_app_entry_type(), test_app_entry_value_unique())
}

#[cfg_attr(tarpaulin, skip)]
pub fn test_sys_entry_value() -> AgentId {
    AgentId::new(&test_key(), &test_key(), &test_key())
}

#[cfg_attr(tarpaulin, skip)]
pub fn test_sys_entry() -> Entry {
    Entry::AgentId(test_sys_entry_value())
}

#[cfg_attr(tarpaulin, skip)]
pub fn test_sys_entry_address() -> Address {
    Address::from("QmWePdZYQrYFBUkBy1GPyCCUf8UmkmptsjtcVqZJ9Tzdse".to_string())
}

#[cfg_attr(tarpaulin, skip)]
pub fn test_unpublishable_entry() -> Entry {
    Entry::Dna(test_dna())
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
