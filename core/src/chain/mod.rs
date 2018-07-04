pub mod entry;
pub mod header;
pub mod memory;
pub mod pair;

use chain::{entry::Entry, pair::Pair};
use serde::{Deserialize, Serialize};
use std;

pub trait SourceChain<'de>: IntoIterator + Serialize + Deserialize<'de> {
    /// append a pair to the source chain if the pair and new chain are both valid, else panic
    fn push(&mut self, &Entry) -> Pair;

    /// returns an iterator referencing pairs from top (most recent) to bottom (genesis)
    fn iter(&self) -> std::slice::Iter<Pair>;

    /// returns true if system and dApp validation is successful
    fn validate(&self) -> bool;

    /// returns a pair for a given header hash
    fn get(&self, k: &str) -> Option<Pair>;

    /// returns a pair for a given entry hash
    fn get_entry(&self, k: &str) -> Option<Pair>;

    /// returns the top (most recent) pair from the source chain
    fn top(&self) -> Option<Pair>;

    /// returns the top (most recent) pair of a given type from the source chain
    fn top_type(&self, t: &str) -> Option<Pair>;
}
