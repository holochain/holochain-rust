use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash as _Hash, Hasher};

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
    fn entry() {
        let e1 = Entry::new(&String::from("bar"));
        let h1 = Header::new(None, &e1);
        let p1 = Pair::new(&h1, &e1);

        assert_eq!(e1, p1.entry());
    }
}
