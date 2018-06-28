use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash as _Hash, Hasher};

use chain::entry::Entry;
use chain::chain::SourceChain;

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
    pub fn new(&chain: SourceChain, entry: &Entry) -> Header {
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

#[cfg(test)]
mod tests {
    use chain::pair::Pair;
    use chain::entry::Entry;
    use chain::header::Header;

    #[test]
    fn header() {
        let e1 = Entry::new(&String::from("foo"));
        let h1 = Header::new(None, &e1);
        let p1 = Pair::new(&h1, &e1);

        assert_eq!(h1, p1.header());
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
