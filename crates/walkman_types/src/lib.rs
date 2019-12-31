extern crate serde;
#[macro_use]
extern crate serde_derive;
// #[macro_use]
// extern crate serde_json;

mod event;

pub const WALKMAN_LOG_PREFIX: &str = "(walkman) ";

pub use event::*;
