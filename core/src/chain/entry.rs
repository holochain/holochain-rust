use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash as _Hash, Hasher};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Entry {
    content: String,
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
        };
        e
    }

    pub fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        _Hash::hash(&self, &mut hasher);
        hasher.finish();
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

    #[test]
    fn new_entry() {
        let c = String::from("foo");
        let e = Entry::new(&c);

        assert_eq!(e.content(), c);
        assert_ne!(e.hash(), 0);
        assert!(e.validate());
    }
}
