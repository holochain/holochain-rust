use ::common::entry::*;
use std::cmp::PartialEq;

pub trait SourceChainInterface {
    fn get(h: Hash) -> Entry;
    fn get_header(h: Hash) -> Hash;
}

#[derive(Clone, Debug, PartialEq)]
pub struct SourceChain {

}

impl SourceChain {
    fn push(e: Entry){}
    //fn serialize() -> str {}
    //fn deseriealize(input: str) {}
}
