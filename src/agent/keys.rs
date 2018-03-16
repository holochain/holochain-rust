use common::entry::Hash;
use std::cmp::PartialEq;

#[derive(Clone, Debug, PartialEq)]
pub struct Key {

}

#[derive(Clone, Debug, PartialEq)]
pub struct Keys {
    pubKey: Key,
    privKey: Key,
    nodeID: Hash
}
