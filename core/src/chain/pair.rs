use chain::entry::Entry;
use chain::header::Header;
use chain::chain::SourceChain;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Pair {
    header: Header,
    entry: Entry,
}

impl Pair {

    pub fn new<'de, C: SourceChain<'de>>(chain: &C, entry_type: String, entry: &Entry) -> Pair {
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

    pub fn header(&self) -> Header {
        self.header.clone()
    }

    pub fn entry(&self) -> Entry {
        self.entry.clone()
    }

    pub fn validate(&self) -> bool {
        self.header.validate() && self.entry.validate() &&
        self.header.entry() == self.entry.hash()
    }

}

#[cfg(test)]
mod tests {
    use super::Pair;
    use chain::entry::Entry;
    use chain::header::Header;
    use chain::memory::MemChain;

    #[test]
    fn new_pair() {
        let chain = MemChain::new();
        let e1 = Entry::new(&String::from("some content"));
        let h1 = Header::new(&chain, "fooType".to_string(), &e1);

        assert_eq!(h1.entry(), e1.hash());
        assert_eq!(h1.next(), None);

        let p1 = Pair::new(&chain, "fooType".to_string(), &e1);
        assert_eq!(e1, p1.entry());
        assert_eq!(h1, p1.header());
    }

    #[test]
    fn validate() {
        let chain = MemChain::new();

        let e1 = Entry::new(&String::from("bar"));
        let p1 = Pair::new(&chain, "fooType".to_string(), &e1);

        assert!(p1.validate());
    }
}
