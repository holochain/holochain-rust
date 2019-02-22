//! Provide a slightly higher-level abstraction over the raw sodium crypto functions for
//! how we are going to be managing keys in Holochain.
//!
#![warn(unused_extern_crates)]

#[macro_use]
extern crate arrayref;

pub mod bundle;
pub mod error;
pub mod keypair;
pub mod util;
