//! Holochain Core Types
//!
//! The reason for having this crate is to have a minimal but
//! complete set of types that are used in most other Holochain
//! crates, but that don't include Holochain itself.
//!
//! Note: This is already quite big. Maybe break the CAS and EAV traits
//! out into their separate crate as well since those are generic and not
//! necessarily bound to Holochain.

extern crate futures;
extern crate multihash;
extern crate rust_base58;
extern crate snowflake;
#[macro_use]
extern crate bitflags;

extern crate serde;
#[macro_use]
extern crate serde_json;
extern crate base64;
extern crate uuid;

#[macro_use]
extern crate serde_derive;
pub mod cas;
pub mod crud_status;
pub mod eav;
pub mod entry;
pub mod error;
pub mod file_validation;
pub mod get_links_args;
pub mod hash;
pub mod json;
pub mod keys;
pub mod signature;
pub mod time;
pub mod validation;
