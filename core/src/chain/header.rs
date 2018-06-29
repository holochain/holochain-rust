use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash as _Hash, Hasher};

use chain::entry::Entry;
use chain::chain::SourceChain;

/// Properties defined in HeadersEntrySchema from golang alpha 1 (hence the title case)
/// @see https://github.com/holochain/holochain-proto/blob/4d1b8c8a926e79dfe8deaa7d759f930b66a5314f/entry_headers.go#L7
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct Header {
    /// the type of this entry
    /// system types may have associated "subconscious" behavior
    Type: String,
    /// ISO8601 time stamp
    Time: String,
    /// link to the immediately preceding header, None is valid only for genesis
    HeaderLink: Option<u64>,
    /// mandatory link to the entry for this header
    EntryLink: u64,
    /// link to the most recent header of the same type, None is valid only for the first of type
    TypeLink: Option<u64>,
    Signature: String,
}

impl _Hash for Header {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.Type.hash(state);
        self.Time.hash(state);
        self.HeaderLink.hash(state);
        self.EntryLink.hash(state);
        self.TypeLink.hash(state);
        self.Signature.hash(state);
    }
}

impl Header {
    pub fn new<'de, C: SourceChain<'de>>(chain: &C, entry_type: String, entry: &Entry) -> Header {
        Header {
            Type: entry_type.clone(),
            // @TODO implement timestamps
            // https://github.com/holochain/holochain-rust/issues/70
            Time: String::new(),
            HeaderLink: chain.top().and_then(|p| Some(p.header().hash())),
            EntryLink: entry.hash(),
            TypeLink: chain.top_type(&entry_type).and_then(|p| Some(p.header().hash())),
            // @TODO implement signatures
            // https://github.com/holochain/holochain-rust/issues/71
            Signature: String::new(),
        }
    }

    pub fn entry_type(&self) -> String {
        self.Type.clone()
    }

    pub fn next(&self) -> Option<u64> {
        self.HeaderLink
    }

    pub fn entry(&self) -> u64 {
        self.EntryLink
    }

    pub fn type_next(&self) -> Option<u64> {
        self.TypeLink
    }

    pub fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        _Hash::hash(&self, &mut hasher);
        hasher.finish()
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
    use chain::memory::MemChain;

    #[test]
    fn header() {
        let chain = MemChain::new();
        let e1 = Entry::new(&String::from("foo"));
        let h1 = Header::new(&chain, "type".to_string(), &e1);
        let p1 = Pair::new(&chain, "type".to_string(), &e1);

        assert_eq!(h1, p1.header());
    }

    #[test]
    fn new_header() {
        let chain = MemChain::new();
        let e = Entry::new(&String::from("foo"));
        let h = Header::new(&chain, "type".to_string(), &e);

        assert_eq!(h.entry(), e.hash());
        assert_eq!(h.next(), None);
        assert_ne!(h.hash(), 0);
        assert!(h.validate());
    }
}
