use common::hash::Hash;

#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    AddPeer(Hash),
}
