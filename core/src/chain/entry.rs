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
        Entry {
            content: content.to_string(),
        }
    }

    pub fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        _Hash::hash(&self, &mut hasher);
        hasher.finish()
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
    /// tests for Entry::new()
    fn new() {
        let c = String::from("foo");
        let e = Entry::new(&c);

        assert_eq!(e.content(), c);
        assert_ne!(e.hash(), 0);
        assert!(e.validate());
    }

    #[test]
    /// tests for entry.hash()
    fn hash() {
        let c = String::from("bar");
        let e = Entry::new(&c);

        assert_eq!(3676438629107045207, e.hash());
    }

    #[test]
    /// tests for entry.content()
    fn content() {
        let c = String::from("baz");
        let e = Entry::new(&c);

        assert_eq!("baz", e.content());
    }

    #[test]
    /// tests for entry.validate()
    fn validate() {
        let c = String::new();
        let e = Entry::new(&c);

        assert!(e.validate());
    }
}
