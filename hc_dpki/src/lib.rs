//! Provide a slightly higher-level abstraction over the raw sodium crypto functions for
//! how we are going to be managing keys in Holochain.
//!
#![warn(unused_extern_crates)]

#[macro_use]
extern crate arrayref;

pub mod error;
pub mod key_blob;
pub mod key_bundle;
pub mod keypair;
pub mod password_encryption;
pub mod utils;

pub const SEED_SIZE: usize = 32;
pub(crate) const SIGNATURE_SIZE: usize = 64;
