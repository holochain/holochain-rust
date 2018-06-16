pub mod memory;

use common::entry::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Pair{
    header: Header,
    entry: Entry,
}

pub trait SourceChain: IntoIterator {
}

// pub struct Pair {
//     pub header: Header,
//     pub entry: Entry,
// }
//
// pub struct SourceChain {
//     pub pairs: Vec<Pair>, // an ordered sequence of headers
//     pub agent: agent::AgentState // a reference back to the agent this chain is for
// }

// impl SourceChain {
    // // returns the latest header
    // pub fn top(self) -> Option<Header> {
    //     self.headers.last()
    // }
    // // returns the nth header
    // pub fn nth(self, i: usize) -> Option<Header> {
    //     self.headers.get(i)
    // }
    // // // returns the top header of a given type
    // // pub fn top_type(self, type: str) -> Option<Header> {
    // //     self.headers.into_iter().filter(|header| header.type == type).last()
    // // }
    // pub fn length(self) -> isize {
    //     self.headers.len()
    // }

    // given a type, agent and entry, adds a generated header and the entry, returns the new header
    // pub fn add_entry(self, type: str, entry: Entry) -> Option<Header> {
    //     let header = Header::fromEntry(self, entry);
    //     self.headers.push(header);
    //     self.entries.insert(header.entry_hash, entry);
    //     header
    // }
    // get a header by its hash
    // pub fn get(self, header_hash: Hash) -> Option<Header> {
    //     self.headers.into_iter().filter(|header| header.hash == header_hash).last()
    // }
    // pub fn get_entry(self, entry_hash: Hash) -> Option<Entry> {
    //     self.entries.get(entry_hash)
    // }
    // pub fn get_entry_header(self, entry_hash: Hash) -> Option<Header> {
    //     self.headers.into_iter().filter(|header| header.entry_hash == entry_hash).last()
    // }
    // walk traverses chain from most recent to first entry calling fn on each one
    // pub fn walk(self, f) -> self {
    //     for h in self.headers.iter().rev() {
    //         f(h, self.get_entry(h.entry_hash))
    //     }
    // }
    // pub fn validate(self) -> Boolean {
    //     true
    // }
    // fn serialize(self) -> str {
    //     // how to handle dot notation for graphvis?
    //     serde_json::to_string(self).unwrap()
    // }
    // fn deserialize(input: str) {
    //     serde_json:from_str(input).unwrap()
    // }
// }
