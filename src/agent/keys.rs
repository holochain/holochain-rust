use common::entry::Hash;
use std::cmp::PartialEq;

#[derive(Clone, Debug, PartialEq)]
pub struct Key {

}

#[derive(Clone, Debug, PartialEq)]
pub struct Keys {
    pub_key: Key,
    priv_key: Key,
    node_id: Hash
}
