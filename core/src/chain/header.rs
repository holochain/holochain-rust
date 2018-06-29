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
    /// agent's cryptographic signature
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
    use chain::SourceChain;
    use chain::entry::Entry;
    use chain::header::Header;
    use chain::memory::MemChain;

    #[test]
    /// tests for Header::new()
    fn new() {
        let chain = MemChain::new();
        let e = Entry::new(&String::from("foo"));
        let h = Header::new(&chain, "type", &e);

        assert_eq!(h.entry(), e.hash());
        assert_eq!(h.next(), None);
        assert_ne!(h.hash(), 0);
        assert!(h.validate());
    }

    #[test]
    /// tests for header.entry_type()
    fn entry_type() {
        let chain = MemChain::new();
        let e = Entry::new(&String::new());
        let h = Header::new(&chain, "foo", &e);

        assert_eq!(h.entry_type(), "foo");
    }

    #[test]
    /// tests for header.time()
    fn time() {
        let chain = MemChain::new();
        let e = Entry::new(&String::new());
        let h = Header::new(&chain, "foo", &e);

        assert_eq!(h.time(), "");
    }

    #[test]
    /// tests for header.next()
    fn next() {
        let mut chain = MemChain::new();
        let t = "foo";

        // first header is genesis so next should be None
        let e1 = Entry::new(&String::new());
        let p1 = chain.push(t, &e1);
        let h1 = p1.header();

        assert_eq!(h1.next(), None);

        // second header next should be first header hash
        let e2 = Entry::new(&String::from("foo"));
        let p2 = chain.push(t, &e2);
        let h2 = p2.header();

        assert_eq!(h2.next(), Some(h1.hash()));
    }

    #[test]
    /// tests for header.entry()
    fn entry() {
        let chain = MemChain::new();
        let t = "foo";

        // header for an entry should contain the entry hash under entry()
        let e = Entry::new(&String::new());
        let h = Header::new(&chain, t, &e);

        assert_eq!(h.entry(), e.hash());
    }

    #[test]
    /// tests for header.type_next()
    fn type_next() {
        let mut chain = MemChain::new();
        let t1 = "foo";
        let t2 = "bar";

        // first header is genesis so next should be None
        let e1 = Entry::new(&String::new());
        let p1 = chain.push(t1, &e1);
        let h1 = p1.header();

        assert_eq!(h1.type_next(), None);

        // second header is a different type so next should be None
        let e2 = Entry::new(&String::new());
        let p2 = chain.push(t2, &e2);
        let h2 = p2.header();

        assert_eq!(h2.type_next(), None);

        // third header is same type as first header so next should be first header hash
        let e3 = Entry::new(&String::new());
        let p3 = chain.push(t1, &e3);
        let h3 = p3.header();

        assert_eq!(h3.type_next(), Some(h1.hash()));
    }

    #[test]
    /// tests for header.signature()
    fn signature() {
        let chain = MemChain::new();
        let t = "foo";

        let e = Entry::new(&String::new());
        let h = Header::new(&chain, t, &e);

        assert_eq!("", h.signature());
    }

    #[test]
    /// tests for header.hash()
    fn hash() {
        let chain = MemChain::new();
        let t1 = "foo";
        let t2 = "bar";

        // basic hash test.
        let e = Entry::new(&String::new());
        let h = Header::new(&chain, t1, &e);

        assert_eq!(6289138340682858684, h.hash());

        // different entries must give different hashes
        let e1 = Entry::new(&String::new());
        let h1 = Header::new(&chain, t1, &e1);

        let e2 = Entry::new(&String::from("a"));
        let h2 = Header::new(&chain, t1, &e2);

        // h and h1 are actually identical so should have the same hash
        assert_eq!(h.hash(), h1.hash());
        assert_ne!(h1.hash(), h2.hash());

        // different types must give different hashes
        let h3 = Header::new(&chain, t2, &e1);
        assert_ne!(h3.hash(), h1.hash());

        // different chain, different hash
        let mut c1 = MemChain::new();

        let p1 = c1.push(t1, &e1);

        // p2 is pushing the same thing as p1, but after p1, so it has a different next val
        let p2 = c1.push(t1, &e1);
        assert_eq!(h1.hash(), p1.header().hash());
        assert_ne!(h1.hash(), p2.header().hash());

        // @TODO is it possible to test that type_next changes the hash in an isolated way?
        // @see https://github.com/holochain/holochain-rust/issues/76
    }

    #[test]
    /// tests for header.validate()
    fn validate() {
        let chain = MemChain::new();
        let t = "foo";

        let e = Entry::new(&String::new());
        let h = Header::new(&chain, t, &e);

        assert!(h.validate());
    }
}
