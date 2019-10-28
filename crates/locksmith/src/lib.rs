#![feature(checked_duration_since)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

mod sync;
pub use sync::*;
