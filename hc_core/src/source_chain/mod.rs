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
}

pub trait SourceChain: IntoIterator {
    fn push(&mut self, &Pair);
    fn iter(&self) -> std::slice::Iter<Pair>;
}
