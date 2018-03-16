use common::entry::Hash;
use std::cmp::PartialEq;

#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    AddPeer(Hash)
}
