/*!
Networking / P2P IPC Abstraction

This crate allows holochain to connect to a running P2P client node over ZeroMq-based socket connection. The recommended ZeroMQ configuration is to use the `ipc:// ` protocol, which will make use of unix domain sockets in a linux or macOs environment. You may need to fall back to `tcp://` for other operating systems.

The main export you should care about is ZmqIpcClient.
*/

#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;
extern crate rmp_serde;
extern crate serde;
extern crate serde_bytes;
#[macro_use]
extern crate serde_derive;
extern crate zmq;

pub mod msg_types;
#[macro_use]
pub mod errors;
mod context;
pub mod message;
mod socket;
mod util;

pub mod ipc_client;
pub use ipc_client::ZmqIpcClient;
