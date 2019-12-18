use crate::{conductor::broadcaster::Broadcaster, interface::Interface};
use crossbeam_channel::Receiver;
use jsonrpc_core::IoHandler;
use jsonrpc_http_server::ServerBuilder;
use std::{net::SocketAddr, thread};

pub struct HttpInterface {
    port: u16,
    bound_address: Option<SocketAddr>,
}

impl HttpInterface {
    pub fn new(port: u16) -> Self {
        HttpInterface {
            port,
            bound_address: None,
        }
    }

    pub fn bound_address(&self) -> Option<SocketAddr> {
        self.bound_address
    }
}

impl Interface for HttpInterface {
    fn run(
        &mut self,
        handler: IoHandler,
        kill_switch: Receiver<()>,
    ) -> Result<(Broadcaster, thread::JoinHandle<()>), String> {
        let url = format!("0.0.0.0:{}", self.port);

        let server = ServerBuilder::new(handler)
            .start_http(&url.parse().expect("Invalid URL!"))
            .map_err(|e| e.to_string())?;
        self.bound_address = Some(*server.address());
        let broadcaster = Broadcaster::Noop;
        let handle = thread::Builder::new()
            .name(format!("http_interface/{}", url))
            .spawn(move || {
                let _ = server; // move `server` into this thread
                let _ = kill_switch.recv();
            })
            .expect("Could not spawn thread for HTTP interface");
        Ok((broadcaster, handle))
    }
}
