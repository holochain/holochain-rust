//! holochain_net is a library that defines an abstract networking layer for
//! different network transports and implements a PeerStore for mapping and
//! managing the topology of transport layers with regard to relay's e.g. for NAT

extern crate base64;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate serde_json;

pub mod error;
pub mod p2p_network;
