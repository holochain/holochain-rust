pub mod api;
pub mod lifecycle;

use holochain_dna::zome::capabilities::ReservedCapabilityNames;

use std::{str::FromStr};

pub trait Defn: FromStr {
    /// return the canonical name of this function definition
    fn as_str(&self) -> &'static str;

    /// convert the canonical name of this function to an index
    fn str_index(s: &str) -> usize;

    /// convert an index to the function definition
    fn from_index(i: usize) -> Self;

    fn capabilities(&self) -> ReservedCapabilityNames;

    // @TODO how to add something to trait that returns functions with unknown params/return?
    // fn as_fn(&self) -> fn(_) -> _;
}
