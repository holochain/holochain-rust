//! Trait system for facilitating non-blocking stream chaining with handshaking
//!
//! # Example
//!
//! ```rust
//! use url2::prelude::*;
//! use in_stream::*;
//!
//! let (send_binding, recv_binding) = crossbeam_channel::unbounded();
//!
//! let server_thread = std::thread::spawn(move || {
//!     let config = TcpBindConfig::default();
//!     let config = TlsBindConfig::new(config).fake_certificate();
//!     let config = WssBindConfig::new(config);
//!     let mut listener:
//!         InStreamListenerWss<InStreamListenerTls<InStreamListenerTcp>> =
//!         InStreamListenerWss::bind(
//!             &url2!("ws://127.0.0.1:0"),
//!             config
//!         ).unwrap();
//!
//!     println!("bound to: {}", listener.binding());
//!     send_binding.send(listener.binding()).unwrap();
//!
//!     let mut srv = loop {
//!         match listener.accept() {
//!             Ok(srv) => break srv,
//!             Err(e) if e.would_block() => std::thread::yield_now(),
//!             Err(e) => panic!("{:?}", e),
//!         }
//!     };
//!
//!     srv.write("hello from server".into()).unwrap();
//!     srv.flush().unwrap();
//!
//!     std::thread::sleep(std::time::Duration::from_millis(100));
//!
//!     let mut res = WsFrame::default();
//!     srv.read(&mut res).unwrap();
//!     assert_eq!("hello from client", res.as_str());
//! });
//!
//! let client_thread = std::thread::spawn(move || {
//!     let binding = recv_binding.recv().unwrap();
//!     println!("connect to: {}", binding);
//!
//!     let mut cli: InStreamWss<InStreamTls<InStreamTcp>> =
//!         InStreamWss::connect(
//!             &binding,
//!             WssConnectConfig::new(
//!                 TlsConnectConfig::new(
//!                     TcpConnectConfig::default()))).unwrap();
//!
//!     cli.write("hello from client".into()).unwrap();
//!     cli.flush().unwrap();
//!
//!     std::thread::sleep(std::time::Duration::from_millis(100));
//!
//!     let mut res = WsFrame::default();
//!     cli.read(&mut res).unwrap();
//!     assert_eq!("hello from server", res.as_str());
//! });
//!
//! server_thread.join().unwrap();
//! client_thread.join().unwrap();
//!
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

mod tls;
pub use tls::*;

mod ws;
pub use ws::*;

pub mod json_rpc;
pub use json_rpc::*;
