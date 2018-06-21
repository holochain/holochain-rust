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
}

pub trait SourceChain: IntoIterator {
    fn push(&mut self, &Pair);
    fn iter(&self) -> std::slice::Iter<Pair>;
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
