//! Trait system for facilitating non-blocking stream chaining with handshaking
//!
//! # Example
//!
//! ```rust
//! use url2::prelude::*;
//! use in_stream::*;
//! ```

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate shrinkwraprs;

mod error;
pub use error::*;

mod std_stream_adapter;
pub use std_stream_adapter::*;

mod r#trait;
pub use r#trait::*;

mod mem;
pub use mem::*;

mod tcp;
pub use tcp::*;

mod tls_certificate;
pub use tls_certificate::*;

mod tls;
pub use tls::*;

/*
mod ws;
pub use ws::*;
*/
