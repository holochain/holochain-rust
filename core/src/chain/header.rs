use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash as _Hash, Hasher};

use chain::entry::Entry;
use chain::SourceChain;

// @TODO - serialize properties as defined in HeadersEntrySchema from golang alpha 1
// @see https://github.com/holochain/holochain-proto/blob/4d1b8c8a926e79dfe8deaa7d759f930b66a5314f/entry_headers.go#L7
// @see https://github.com/holochain/holochain-rust/issues/75
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Header {
    /// the type of this entry
    /// system types may have associated "subconscious" behavior
    entry_type: String,
    /// ISO8601 time stamp
    time: String,
    /// link to the immediately preceding header, None is valid only for genesis
    next: Option<u64>,
    /// mandatory link to the entry for this header
    entry: u64,
    /// link to the most recent header of the same type, None is valid only for the first of type
    type_next: Option<u64>,
    signature: String,
}

impl _Hash for Header {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.entry_type.hash(state);
        self.time.hash(state);
        self.next.hash(state);
        self.entry.hash(state);
        self.type_next.hash(state);
        self.signature.hash(state);
    }
}

impl Header {
    pub fn new<'de, C: SourceChain<'de>>(chain: &C, entry_type: &str, entry: &Entry) -> Header {
        Header {
            entry_type: entry_type.to_string(),
            // @TODO implement timestamps
            // https://github.com/holochain/holochain-rust/issues/70
            time: String::new(),
            next: chain.top().and_then(|p| Some(p.header().hash())),
            entry: entry.hash(),
            type_next: chain
                .top_type(entry_type)
                .and_then(|p| Some(p.header().hash())),
            // @TODO implement signatures
            // https://github.com/holochain/holochain-rust/issues/71
            signature: String::new(),
        }
    }

    pub fn entry_type(&self) -> String {
        self.entry_type.clone()
    }

    pub fn time(&self) -> String {
        self.time.clone()
    }

    pub fn next(&self) -> Option<u64> {
        self.next
    }

    pub fn entry(&self) -> u64 {
        self.entry
    }

    pub fn type_next(&self) -> Option<u64> {
        self.type_next
    }

    pub fn signature(&self) -> String {
        self.signature.clone()
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
    use chain::entry::Entry;
    use chain::header::Header;
    use chain::memory::MemChain;
    use chain::pair::Pair;

    #[test]
    fn header() {
        let chain = MemChain::new();
        let e1 = Entry::new(&String::from("foo"));
        let h1 = Header::new(&chain, "type", &e1);
        let p1 = Pair::new(&chain, "type", &e1);

        assert_eq!(h1, p1.header());
    }

    #[test]
    fn new_header() {
        let chain = MemChain::new();
        let e = Entry::new(&String::from("foo"));
        let h = Header::new(&chain, "type", &e);

        assert_eq!(h.entry(), e.hash());
        assert_eq!(h.next(), None);
        assert_ne!(h.hash(), 0);
        assert!(h.validate());
    }
}
