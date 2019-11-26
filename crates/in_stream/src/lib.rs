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
//!     let mut listener: InStreamListenerWssType = InStreamListenerWss::bind(
//!         &url2!("wss://0.0.0.0:0"),
//!         WssBindConfig::default()
//!             // for unit test we want to use a fake certificate
//!             // because it's slow to generate an RSA keypair
//!             .sub_bind_config(TlsBindConfig::with_fake_certificate()),
//!     ).unwrap();
//!
//!     send_binding.send(listener.binding()).unwrap();
//!
//!     // we have a partial websocket that still needs to handshake
//!     let mut srv_wss_partial = listener.accept_blocking().unwrap();
//!
//!     // we process it to get a full stream
//!     let mut srv_wss = srv_wss_partial.process_blocking().unwrap();
//!
//!     srv_wss.write_frame(b"hello from server".to_vec()).unwrap();
//!
//!     loop {
//!         std::thread::sleep(std::time::Duration::from_millis(1));
//!
//!         match srv_wss.read_frame::<Vec<u8>>() {
//!             Ok(frame) => {
//!                 assert_eq!(
//!                     "hello from client",
//!                     &String::from_utf8_lossy(&frame),
//!                 );
//!                 break;
//!             }
//!             Err(e) if e.would_block() => (),
//!             Err(e) => panic!("{:?}", e),
//!         }
//!     }
//! });
//!
//! let client_thread = std::thread::spawn(move || {
//!     let binding = recv_binding.recv().unwrap();
//!
//!     let mut cli_wss_partial: InStreamPartialWssType = InStreamPartialWss::connect(
//!         &binding,
//!         Default::default(),
//!     ).unwrap();
//!
//!     let mut cli_wss = cli_wss_partial.process_blocking().unwrap();
//!
//!     cli_wss.write_frame(b"hello from client".to_vec()).unwrap();
//!
//!     loop {
//!         std::thread::sleep(std::time::Duration::from_millis(1));
//!
//!         match cli_wss.read_frame::<Vec<u8>>() {
//!             Ok(frame) => {
//!                 assert_eq!(
//!                     "hello from server",
//!                     &String::from_utf8_lossy(&frame),
//!                 );
//!                 break;
//!             }
//!             Err(e) if e.would_block() => (),
//!             Err(e) => panic!("{:?}", e),
//!         }
//!     }
//! });
//!
//! server_thread.join().unwrap();
//! client_thread.join().unwrap();
//! ```

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate shrinkwraprs;

mod error;
pub use error::*;

mod r#trait;
pub use r#trait::*;

mod mem;
pub use mem::*;

mod tcp;
pub use tcp::*;

mod tcp2;
pub use tcp2::*;

mod tls_certificate;
pub use tls_certificate::*;

mod tls;
pub use tls::*;

mod ws;
pub use ws::*;
