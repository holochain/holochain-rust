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
pub mod context;
pub mod message;
mod util;

mod ipc_client;
pub use ipc_client::IpcClient;
