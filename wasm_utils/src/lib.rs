//! Library holding necesarray code for the Ribosome and that is also useful for hdk-rust,
//! or more generally for making rust code that the Ribosome will run.
//! Must not have any dependency with other Holochain crates.
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

pub mod error;
pub mod memory_allocation;
pub mod memory_serialization;
