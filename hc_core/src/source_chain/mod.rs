pub mod memory;
use std;

use common::entry::Entry;
use common::entry::Header;

#[derive(Clone, Debug, PartialEq)]
pub struct Pair {
    header: Header,
    entry: Entry,
}

impl Pair {
    pub fn new(header: &Header, entry: &Entry) -> Pair {
        Pair {
            header: header.clone(),
            entry: entry.clone(),
        }
    }

    pub fn header(&self) -> Header {
        self.header.clone()
    }

    pub fn entry(&self) -> Entry {
        self.entry.clone()
    }

    pub fn validate(&self) -> bool {
        self.header.validate() && self.entry.validate()
    }
}

pub trait SourceChain: IntoIterator {
    // appends the given pair to the source chain, if doing so results in a new valid chain
    // returns the potentially updated chain
    fn push(&mut self, &Pair);
    fn iter(&self) -> std::slice::Iter<Pair>;
    fn validate(&self) -> bool;
    fn get(&self, k: u64) -> Option<Pair>;
    fn get_entry(&self, k:u64) -> Option<Pair>;
}

#[cfg(test)]
mod tests {
    use super::Pair;
    use common::entry::Entry;
    use common::entry::Header;

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
}
