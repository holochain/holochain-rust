use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash as _Hash, Hasher};
use source_chain::SourceChain;
use source_chain::Pair;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Entry {
    content: String,
    hash: u64,
}

impl _Hash for Entry {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.content.hash(state);
    }
}

impl Entry {
    pub fn new(content: &str) -> Entry {
        let mut e = Entry {
            content: content.to_string(),
            hash: 0,
        };
        let mut hasher = DefaultHasher::new();
        _Hash::hash(&e, &mut hasher);
        e.hash = hasher.finish();
        e
    }

    pub fn hash(&self) -> u64 {
        self.hash
    }

    pub fn content(&self) -> String {
        self.content.clone()
    }

    pub fn validate(&self) -> bool {
        // always valid iff immutable and new() enforces validity
        true
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Header {
    /// the type of this entry
    /// system types may have associated "subconscious" behavior
    entry_type: String,
    /// optional link to the immediately preceding header in the chain
    next: Option<u64>,
    /// mandatory link to the entry for this header
    entry: u64,
    /// optional link to the most recent header of the same type in the chain
    next_of_type: Option<u64>,
    hash: u64,
}

impl _Hash for Header {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.header_link.hash(state);
        self.entry_link.hash(state);
        self.entry_type.hash(state);
        self.type_link.hash(state);
    }
}

impl Header {
    pub fn new(&chain: SourceChain<Pair>, entry: &Entry) -> Header {
        let previous = chain.top();
        let mut h = Header {
            previous,
            entry: entry.hash(),
            hash: 0,
        };
        let mut hasher = DefaultHasher::new();
        _Hash::hash(&h, &mut hasher);
        h.hash = hasher.finish();
        h
    }

    pub fn entry(&self) -> u64 {
        self.entry_link
    }

    pub fn next(&self) -> Option<u64> {
        self.header_link
    }

    pub fn hash(&self) -> u64 {
        self.hash
    }

    pub fn validate(&self) -> bool {
        // always valid iff immutable and new() enforces validity
        true
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Hash {}

#[cfg(test)]
mod tests {
    use super::Entry;
    use super::Header;

    #[test]
    fn new_entry() {
        let c = String::from("foo");
        let e = Entry::new(&c);

        assert_eq!(e.content(), c);
        assert_ne!(e.hash(), 0);
        assert!(e.validate());
    }

    #[test]
    fn new_header() {
        let e = Entry::new(&String::from("foo"));
        let h = Header::new(None, &e);

        assert_eq!(h.entry(), e.hash());
        assert_eq!(h.next(), None);
        assert_ne!(h.hash(), 0);
        assert!(h.validate());
    }

}
