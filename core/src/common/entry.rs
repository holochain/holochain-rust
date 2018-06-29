use std::{
    collections::hash_map::DefaultHasher, hash::{Hash as _Hash, Hasher},
};

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
    // these are hashes instead of references so that they can be serialized/validated as data in
    // any/all implementations
    previous: Option<u64>,
    entry: u64,
    hash: u64,
}

impl _Hash for Header {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.previous.hash(state);
        self.entry.hash(state);
    }
}

impl Header {
    pub fn new(previous: Option<u64>, entry: &Entry) -> Header {
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
        self.entry
    }

    pub fn previous(&self) -> Option<u64> {
        self.previous
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
    use super::{Entry, Header};

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
        assert_eq!(h.previous(), None);
        assert_ne!(h.hash(), 0);
        assert!(h.validate());
    }

}
