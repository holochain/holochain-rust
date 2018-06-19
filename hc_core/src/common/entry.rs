use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash as _Hash, Hasher};

#[derive(Clone, Debug, PartialEq)]
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
    pub fn new (content: &str) -> Entry {
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
}

#[derive(Clone, Debug, PartialEq)]
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
}

#[derive(Clone, Debug, PartialEq)]
pub struct Hash {}
