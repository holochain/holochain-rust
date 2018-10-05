//! Library holding necessary code for the Ribosome  that is also useful for hdk-rust,
//! or more generally for making rust code that the Ribosome can run.
//! Must not have any dependency with any other Holochain crates.
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

pub mod error;
pub mod memory_allocation;
pub mod memory_serialization;
pub mod validation;