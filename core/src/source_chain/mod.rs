pub mod memory;

use common::entry::{Entry, Header};
use serde::{Deserialize, Serialize};
use std;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Pair {
    header: Header,
    entry: Entry,
}

impl Pair {
    pub fn new(header: &Header, entry: &Entry) -> Pair {
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
        self.header.validate() && self.entry.validate() && self.header.entry() == self.entry.hash()
    }
}

pub trait SourceChain<'de>: IntoIterator + Serialize + Deserialize<'de> {
    /// append a pair to the source chain if the pair and new chain are both valid, else panic
    fn push(&mut self, &Pair);

    /// returns an iterator referencing pairs from top (most recent) to bottom (genesis)
    fn iter(&self) -> std::slice::Iter<Pair>;

    /// returns true if system and dApp validation is successful
    fn validate(&self) -> bool;

    /// returns a pair for a given header hash
    fn get(&self, k: u64) -> Option<Pair>;

    /// returns a pair for a given entry hash
    fn get_entry(&self, k: u64) -> Option<Pair>;
}

#[cfg(test)]
mod tests {
    use super::Pair;
    use common::entry::{Entry, Header};

    #[test]
    fn new_pair() {
        let e1 = Entry::new(&String::from("some content"));
        let h1 = Header::new(None, &e1);
        assert_eq!(h1.entry(), e1.hash());
        assert_eq!(h1.previous(), None);

        let p1 = Pair::new(&h1, &e1);
        assert_eq!(e1, p1.entry());
        assert_eq!(h1, p1.header());
    }

    #[test]
    fn header() {
        let e1 = Entry::new(&String::from("foo"));
        let h1 = Header::new(None, &e1);
        let p1 = Pair::new(&h1, &e1);

        assert_eq!(h1, p1.header());
    }

    #[test]
    fn entry() {
        let e1 = Entry::new(&String::from("bar"));
        let h1 = Header::new(None, &e1);
        let p1 = Pair::new(&h1, &e1);

        assert_eq!(e1, p1.entry());
    }

    #[test]
    fn validate() {
        let e1 = Entry::new(&String::from("bar"));
        let h1 = Header::new(None, &e1);
        let p1 = Pair::new(&h1, &e1);

        assert!(p1.validate());
    }

    #[test]
    #[should_panic(expected = "attempted to create an invalid pair")]
    fn invalidate() {
        let e1 = Entry::new(&String::from("foo"));
        let e2 = Entry::new(&String::from("bar"));
        let h1 = Header::new(None, &e1);

        // header/entry mismatch, must panic!
        Pair::new(&h1, &e2);
    }
}
