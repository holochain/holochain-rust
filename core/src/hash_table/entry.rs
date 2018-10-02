use hash_table::sys_entry::EntryType;
use cas::content::Address;
use cas::content::AddressableContent;
use cas::content::Content;
use std::fmt;

/// integrates Entry Content with the Rust type system
#[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq)]
pub struct Entry(Content);

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for Entry {
    fn from(s: String) -> Entry {
        Entry(s)
    }
}

impl AddressableContent for Entry {
    fn content(&self) -> Content {
        self.0
    }

    fn from_content(c: &Content) -> Self {
        Entry::from(&c)
    }
}

impl Entry {
    pub fn new() -> Entry {
        Entry("".to_string())
    }
}

#[derive(Clone, PartialEq, Debug, Hash)]
pub struct EntryHeader {
    entry_type: EntryType,
    entry_address: Address,
}

impl EntryHeader {
    pub fn new(entry_type: &EntryType, entry_address: &Address) -> EntryHeader {
        EntryHeader {
            entry_type: entry_type.clone(),
            entry_address: entry_address.clone(),
        }
    }

    pub fn entry_type(&self) -> EntryType {
        self.entry_type.clone()
    }

    pub fn entry_address(&self) -> Address {
        self.entry_address.clone()
    }

    /// returns true if the entry type is a system entry
    pub fn is_sys(&self) -> bool {
        &self.entry_type != EntryType::App
    }

    /// returns true if the entry type is an app entry
    pub fn is_app(&self) -> bool {
        &self.entry_type == EntryType::App
    }
}

#[cfg(test)]
pub mod tests {
    use hash::HashString;
    use hash_table::{entry::Entry, sys_entry::EntryType};
    use json::{FromJson, ToJson};
    use key::Key;
    use snowflake;

    /// dummy entry type
    pub fn test_type() -> String {
        "testEntryType".into()
    }

    /// dummy entry type, same as test_type()
    pub fn test_type_a() -> String {
        test_type()
    }

    /// dummy entry type, differs from test_type()
    pub fn test_type_b() -> String {
        "testEntryTypeB".into()
    }

    /// dummy entry content
    pub fn test_content() -> String {
        "test entry content".into()
    }

    /// dummy entry content, same as test_content()
    pub fn test_content_a() -> String {
        test_content()
    }

    /// dummy entry content, differs from test_content()
    pub fn test_content_b() -> String {
        "other test entry content".into()
    }

    /// dummy entry
    pub fn test_entry() -> Entry {
        Entry::new(&test_type(), &test_content())
    }

    /// the correct hash for test_entry()
    pub fn test_entry_hash() -> HashString {
        HashString::from("QmbXSE38SN3SuJDmHKSSw5qWWegvU7oTxrLDRavWjyxMrT".to_string())
    }

    /// dummy entry, same as test_entry()
    pub fn test_entry_a() -> Entry {
        test_entry()
    }

    /// dummy entry, differs from test_entry()
    pub fn test_entry_b() -> Entry {
        Entry::new(&test_type_b(), &test_content_b())
    }

    /// dummy entry with unique string content
    pub fn test_entry_unique() -> Entry {
        Entry::new(&test_type(), &snowflake::ProcessUniqueId::new().to_string())
    }

    #[test]
    /// tests for PartialEq
    fn eq() {
        let c1 = "foo";
        let c2 = "bar";
        let t1 = "a";
        let t2 = "b";

        // same type and content is equal
        assert_eq!(Entry::new(t1, c1), Entry::new(t1, c1));

        // same type different content is not equal
        assert_ne!(Entry::new(t1, c1), Entry::new(t1, c2));

        // same content different type is equal
        // @see https://github.com/holochain/holochain-rust/issues/85
        assert_eq!(Entry::new(t1, c1), Entry::new(t2, c1));

        // different content different type is not equal
        assert_ne!(Entry::new(t1, c1), Entry::new(t2, c2));
    }

    #[test]
    /// tests that hash equality matches PartialEq
    fn eq_hash() {
        let c1 = "foo";
        let c2 = "bar";
        let t1 = "a";
        let t2 = "b";

        // same type and content is equal
        assert_eq!(Entry::new(t1, c1).hash(), Entry::new(t1, c1).hash());

        // same type different content is not equal
        assert_ne!(Entry::new(t1, c1).hash(), Entry::new(t1, c2).hash());

        // same content different type is equal
        // @see https://github.com/holochain/holochain-rust/issues/85
        assert_eq!(Entry::new(t1, c1).hash(), Entry::new(t2, c1).hash());

        // different content different type is not equal
        assert_ne!(Entry::new(t1, c1).hash(), Entry::new(t2, c2).hash());
    }

    #[test]
    /// tests for Entry::new()
    fn new() {
        let c = "foo";
        let t = "bar";
        let e = Entry::new(t, c);

        assert_eq!(e.content(), c);
        assert_ne!(e.hash(), HashString::new());
        assert!(e.validate());
    }

    #[test]
    /// test entry.hash() against a known value
    fn hash_known() {
        assert_eq!(test_entry_hash(), test_entry().hash());
    }

    #[test]
    /// test that the content changes the hash
    fn hash_content() {
        let t = "bar";
        let c1 = "baz";
        let c2 = "foo";

        let e1 = Entry::new(t, c1);
        let e2 = Entry::new(t, c1);
        let e3 = Entry::new(t, c2);

        // same content same hash
        assert_eq!(e1.hash(), e2.hash());

        // different content, different hash
        assert_ne!(e1.hash(), e3.hash());
    }

    #[test]
    /// test that the entry type does NOT change the hash
    fn hash_entry_type() {
        let t1 = "barType";
        let t2 = "fooo";
        let c = "barr";

        let e1 = Entry::new(t1, c);
        let e2 = Entry::new(t2, c);

        assert_eq!(e1.hash(), e2.hash());
    }

    #[test]
    /// tests for entry.content()
    fn content() {
        let c = "baz";
        let t = "foo";
        let e = Entry::new(t, c);

        assert_eq!("baz", e.content());
    }

    #[test]
    /// tests for entry.entry_type()
    fn entry_type() {
        let t = "bar";
        let c = "foo";
        let e = Entry::new(t, c);

        assert_eq!(t, e.entry_type());
    }

    #[test]
    /// tests for entry.validate()
    fn validate() {
        let t = "";
        let c = "";
        let e = Entry::new(t, c);

        assert!(e.validate());
    }

    #[test]
    /// tests for entry.key()
    fn test_key() {
        assert_eq!(test_entry().hash(), test_entry().key());
    }

    #[test]
    /// test that we can round trip through JSON
    fn json_round_trip() {
        let e = test_entry_a();
        let expected = r#"{"content":"test entry content","entry_type":"testEntryType"}"#;
        assert_eq!(expected, e.to_json().unwrap());
        assert_eq!(e, Entry::from_json(expected).unwrap());
        assert_eq!(e, Entry::from_json(&e.to_json().unwrap()).unwrap());
    }

    #[test]
    /// test that we can detect system entry types
    fn is_sys() {
        for sys_type in vec![
            EntryType::AgentId,
            EntryType::Deletion,
            EntryType::Dna,
            EntryType::Header,
            EntryType::Key,
            EntryType::Link,
            EntryType::Migration,
        ] {
            let entry = Entry::new(sys_type.as_str(), "");
            assert!(entry.is_sys());
            assert!(!entry.is_app());
        }
    }

    #[test]
    /// test that we can detect app entry types
    fn is_app() {
        let entry = Entry::new("foo", "");
        assert!(entry.is_app());
        assert!(!entry.is_sys());
    }
}
