use chain::{entry::Entry, header::Header, SourceChain};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Pair {
    header: Header,
    entry: Entry,
}

impl Pair {
    /// build a new Pair from a chain and entry
    /// Header is generated automatically
    /// a Pair is immutable, but the chain is mutable if chain.push() is used.
    /// this means that if two Pairs X and Y are generated for chain C then Pair X is pushed onto
    /// C to create chain C' (containing X), then Pair Y is no longer valid as the headers would
    /// need to include X. Pair Y can be regenerated with the same parameters as Y' and will be
    /// now be valid, the new Y' will include correct headers pointing to X.
    /// @see chain::entry::Entry
    /// @see chain::header::Header
    pub fn new<'de, C: SourceChain<'de>>(chain: &C, entry_type: &str, entry: &Entry) -> Pair {
        let header = Header::new(chain, entry_type, entry);

        let p = Pair {
            header: header.clone(),
            entry: entry.clone(),
        };

        if !p.validate() {
            // we panic as no code path should attempt to create invalid pairs
            panic!("attempted to create an invalid pair");
        };

        p
    }

    /// header getter
    pub fn header(&self) -> Header {
        self.header.clone()
    }

    /// entry getter
    pub fn entry(&self) -> Entry {
        self.entry.clone()
    }

    /// true if the pair is valid
    pub fn validate(&self) -> bool {
        self.header.validate() && self.entry.validate() && self.header.entry() == self.entry.hash()
    }
}

#[cfg(test)]
mod tests {
    use super::Pair;
    use chain::{entry::Entry, header::Header, memory::MemChain, SourceChain};

    #[test]
    /// tests for Pair::new()
    fn new() {
        let chain = MemChain::new();
        let e1 = Entry::new(&String::from("some content"));
        let h1 = Header::new(&chain, "fooType", &e1);

        assert_eq!(h1.entry(), e1.hash());
        assert_eq!(h1.next(), None);

        let p1 = Pair::new(&chain, "fooType", &e1);
        assert_eq!(e1, p1.entry());
        assert_eq!(h1, p1.header());
    }

    #[test]
    /// tests for pair.header()
    fn header() {
        let chain = MemChain::new();
        let t = "foo";
        let e = Entry::new(&String::from("foo"));
        let h = Header::new(&chain, t, &e);
        let p = Pair::new(&chain, t, &e);

        assert_eq!(h, p.header());
    }

    #[test]
    /// tests for pair.entry()
    fn entry() {
        let mut chain = MemChain::new();
        let t = "foo";
        let e = Entry::new(&String::new());
        let p = chain.push(t, &e);

        assert_eq!(e, p.entry());
    }

    #[test]
    /// tests for pair.validate()
    fn validate() {
        let chain = MemChain::new();

        let e1 = Entry::new(&String::from("bar"));
        let p1 = Pair::new(&chain, "fooType", &e1);

        assert!(p1.validate());
    }
}
