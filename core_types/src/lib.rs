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
extern crate riker;
extern crate rust_base58;
extern crate serde;
extern crate serde_json;
extern crate snowflake;
extern crate walkdir;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate unwrap_to;
extern crate riker_default;
extern crate riker_patterns;
#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate serde_derive;
pub mod actor;
pub mod cas;
pub mod chain_header;
pub mod crud_status;
pub mod eav;
pub mod entry;
pub mod entry_meta;
pub mod entry_type;
pub mod error;
pub mod get_links_args;
pub mod hash;
pub mod json;
pub mod keys;
pub mod links_entry;
pub mod to_entry;
pub mod validation;
