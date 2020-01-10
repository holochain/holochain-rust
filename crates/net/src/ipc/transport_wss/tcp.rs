//! abstraction for working with Websocket connections
//! TcpStream specific functions

use crate::ipc::transport_wss::TransportWss;

[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_NET)]
impl TransportWss<std::net::TcpStream> {
    /// convenience constructor for creating a websocket "Transport"
    /// instance that is based of the rust std TcpStream
    pub fn with_std_tcp_stream() -> Self {
        TransportWss::new(|uri| {
            let socket = std::net::TcpStream::connect(uri)?;
            socket.set_nonblocking(true)?;
            Ok(socket)
        })
    }
}
