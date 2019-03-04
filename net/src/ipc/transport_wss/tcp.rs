//! abstraction for working with Websocket connections
//! TcpStream specific functions

use crate::ipc::{
    transport::{Transport, TransportError, TransportEvent, TransportId, TransportResult},
    transport_wss::{TransportWss, DEFAULT_HEARTBEAT_WAIT_MS},
};

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

    /// connect and wait for a Connect event response
    pub fn wait_connect(&mut self, uri: &str) -> TransportResult<TransportId> {
        // Launch connection attempt
        let transport_id = self.connect(&uri)?;
        // Wait for a successful response
        let mut out = Vec::new();
        let start = std::time::Instant::now();
        while (start.elapsed().as_millis() as usize) < DEFAULT_HEARTBEAT_WAIT_MS {
            let (_did_work, evt_lst) = self.poll()?;
            for evt in evt_lst {
                match evt {
                    TransportEvent::Connect(id) => {
                        if id == transport_id {
                            return Ok(id);
                        }
                    }
                    _ => out.push(evt),
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(3));
        }
        // Timed out
        Err(TransportError::new(format!(
            "ipc wss connection attempt timed out for '{}'. Received events: {:?}",
            transport_id, out
        )))
    }
}
