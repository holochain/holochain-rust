use error::HolochainError;
use hash::HashString;
use hash_table::sys_entry::EntryType;
use json::{FromJson, ToJson};
use key::Key;
use multihash::Hash;
use serde_json;
use std::{
    hash::{Hash as StdHash, Hasher},
    str::FromStr,
};

/// Structure holding actual data in a source chain "Item"
/// data is stored as a JSON string
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entry {
    content: String,

    // @TODO do NOT serialize entry_type in Entry as it should only be in Header
    // @see https://github.com/holochain/holochain-rust/issues/80
    entry_type: String,
}

impl PartialEq for Entry {
    fn eq(&self, other: &Entry) -> bool {
        // @TODO is this right?
        // e.g. two entries with the same content but different type are equal
        // @see https://github.com/holochain/holochain-rust/issues/85
        self.hash() == other.hash()
    }
}

/// implement Hash for Entry to match PartialEq logic
// @TODO is this right?
// e.g. two entries with the same content but different type are equal
// @see https://github.com/holochain/holochain-rust/issues/85
impl StdHash for Entry {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.content.hash(state);
    }
}

impl Entry {
    /// build a new Entry from passed content
    /// an Entry is immutable, this is important for absolutely everything downstream
    /// an entry is not valid until paired with a header and included in a chain.
    /// @see chain::header::Header
    /// @see chain::pair::Pair
    pub fn new(entry_type: &str, content: &str) -> Entry {
        Entry {
            entry_type: entry_type.to_string(),
            content: content.to_string(),
        }
    }

    /// hashes the entry
    pub fn hash(&self) -> HashString {
        // @TODO - this is the wrong string being hashed
        // @see https://github.com/holochain/holochain-rust/issues/103
        let string_to_hash = &self.content;

        // @TODO the hashing algo should not be hardcoded
        // @see https://github.com/holochain/holochain-rust/issues/104
        HashString::encode_from_str(string_to_hash, Hash::SHA2256)
    }

    /// content getter
    pub fn content(&self) -> String {
        self.content.clone()
    }

    /// entry_type getter
    pub fn entry_type(&self) -> String {
        self.entry_type.clone()
    }

    /// returns true if the entry type is a system entry
    pub fn is_sys(&self) -> bool {
        EntryType::from_str(&self.entry_type).unwrap() != EntryType::App
    }

    /// returns true if the entry type is an app entry
    pub fn is_app(&self) -> bool {
        EntryType::from_str(&self.entry_type).unwrap() == EntryType::App
    }
}

impl Key for Entry {
    fn key(&self) -> HashString {
        self.hash()
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
