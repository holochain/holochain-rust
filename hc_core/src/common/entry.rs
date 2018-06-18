use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash as _Hash, Hasher};

#[derive(Clone, Debug, PartialEq, Hash)]
pub struct Entry {
    content: String,
}

impl Entry {
    pub fn new (content: &String) -> Entry {
        Entry {
            content: content.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Header {
    previous: Option<String>,
    entry: Entry,
    hash: u64,
}

impl _Hash for Header {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.previous.hash(state);
        self.entry.hash(state);
    }
}

impl Header {
    pub fn new(previous: Option<&String>, entry: &Entry) -> Header {
        let mut h = Header {
            previous: match previous {
                Some(p) => Some(p.clone()),
                None => None,
            },
            entry: entry.clone(),
            hash: 0,
        };
        let mut hasher = DefaultHasher::new();
        h.hash(&mut hasher);
        h.hash = hasher.finish();
        h
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Hash {}
