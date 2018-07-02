use std::{
    collections::hash_map::DefaultHasher, hash::{Hash as _Hash, Hasher},
};

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
    /// build a new Entry from passed content
    /// an Entry is immutable, this is important for absolutely everything downstream
    /// an entry is not valid until paired with a header and included in a chain.
    /// @see chain::header::Header
    /// @see chain::pair::Pair
    pub fn new(content: &str) -> Entry {
        Entry {
            content: content.to_string(),
        }
    }

    /// hashes the entry
    pub fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        _Hash::hash(&self, &mut hasher);
        hasher.finish()
    }

    /// content getter
    pub fn content(&self) -> String {
        self.content.clone()
    }

    /// returns true if the entry is valid
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
        let c1 = String::from("bar");
        let e1 = Entry::new(&c1);

        assert_eq!(3676438629107045207, e1.hash());

        // same content, same hash
        let c2 = String::from("bar");
        let e2 = Entry::new(&c2);

        assert_eq!(e1.hash(), e2.hash());

        // different content, different hash
        let c3 = String::from("foo");
        let e3 = Entry::new(&c3);

        assert_ne!(e1.hash(), e3.hash());
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
