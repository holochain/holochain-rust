//! Holochain Core Types
//!
//! The reason for having this crate is to have a minimal but
//! complete set of types that are used in most other Holochain
//! crates, but that don't include Holochain itself.
#![feature(try_trait)]
#![feature(never_type)]
#![feature(checked_duration_since)]
#![warn(unused_extern_crates)]

extern crate backtrace;
extern crate base64;
extern crate chrono;
extern crate futures;
#[macro_use]
extern crate lazy_static;
extern crate multihash;
extern crate serde;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate snowflake;
// #[macro_use] extern crate shrinkwraprs;
#[macro_use]
extern crate holochain_json_derive;
extern crate holochain_json_api;
extern crate holochain_locksmith;
extern crate holochain_persistence_api;
extern crate lib3h_crypto_api;
extern crate regex;
#[cfg(test)]
#[macro_use]
extern crate maplit;
// #[macro_use]
// extern crate shrinkwraprs;
extern crate hcid;
extern crate wasmi;
#[macro_use]
extern crate log;

pub mod chain_header;
pub mod crud_status;
pub mod eav;
pub mod entry;
pub mod error;
#[macro_use]
extern crate objekt;
pub mod agent;
pub mod bits_n_pieces;
pub mod chain_migrate;
pub mod dna;
pub mod hdk_version;
pub mod link;
pub mod network;
pub mod signature;
pub mod time;
pub mod ugly;
pub mod validation;
#[macro_use]
extern crate holochain_logging;

pub const HDK_HASH: &str = env!(
    "HDK_HASH",
    "failed to obtain HDK hash from build environment. Check build.rs"
);

// may be empty because code isn't always built from git repo
pub const GIT_HASH: &str = env!(
    "GIT_HASH",
    ""
);

#[cfg(test)]
mod test_hash {
    use super::*;

    #[test]
    fn test_hash() {
        assert_eq!(HDK_HASH.chars().count(), 32); // Nix MD5 hash
        assert!(
            HDK_HASH.is_ascii(),
            "HDK_HASH contains non-ascii characters"
        );
    }
}
