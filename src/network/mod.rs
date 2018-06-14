use common::entry::Hash;

#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    AddPeer(Hash)
}
