//! Holochain Core Types
//!
//! The reason for having this crate is to have a minimal but
//! complete set of types that are used in most other Holochain
//! crates, but that don't include Holochain itself.
//!
//! Note: This is already quite big. Maybe break the CAS and EAV traits
//! out into their separate crate as well since those are generic and not
//! necessarily bound to Holochain.
#![feature(try_from)]
#![feature(never_type)]
#![feature(trait_alias)]

#[macro_use]
extern crate arrayref;
extern crate base64;
extern crate chrono;
extern crate futures;
#[macro_use]
extern crate lazy_static;
extern crate multihash;
extern crate reed_solomon;
extern crate rust_base58;
extern crate serde;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate snowflake;
#[macro_use]
extern crate holochain_core_types_derive;
extern crate regex;
#[cfg(test)]
#[macro_use]
extern crate maplit;

extern crate uuid;

pub mod cas;
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
pub mod hash;
pub mod json;
pub mod link;
pub mod signature;
pub mod time;
pub mod validation;
