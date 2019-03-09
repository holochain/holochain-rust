//! Provide a slightly higher-level abstraction over the raw sodium crypto functions for
//! how we are going to be managing keys in Holochain.
//!
#![warn(unused_extern_crates)]

#[macro_use]
extern crate lazy_static;

pub const CONTEXT_SIZE: usize = 8;
pub const SEED_SIZE: usize = 32;
pub(crate) const SIGNATURE_SIZE: usize = 64;

lazy_static! {
    pub static ref CODEC_HCS0: hcid::HcidEncoding =
        hcid::with_hcs0().expect("HCID failed miserably with hcs0.");
    pub static ref CODEC_HCK0: hcid::HcidEncoding =
        hcid::with_hck0().expect("HCID failed miserably with_hck0.");
}

pub mod key_blob;
pub mod key_bundle;
pub mod keypair;
pub mod password_encryption;
pub mod seed;
pub mod utils;
