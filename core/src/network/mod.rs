pub mod message;

#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    AddPeer(String),
}
