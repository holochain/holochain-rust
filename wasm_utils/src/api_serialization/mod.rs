/// This module holds structs for all arguments and return types
/// that get serialized and deserialized between core native and
/// the WASM based ribosome.
///
/// When these types get changed their counter parts in all HDKs
/// have to change with them! Otherwise we get deserialization
/// errors in the ribosome.
///
/// For the case of HDK-rust we can use the exact same types by
/// importing this module.
pub mod commit;
pub mod get_entry;
pub mod get_links;
pub mod validation;
mod zome_api_globals;

pub use self::zome_api_globals::*;
