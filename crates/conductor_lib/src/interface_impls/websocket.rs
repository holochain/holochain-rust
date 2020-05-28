use crate::{conductor::broadcaster::Broadcaster, interface::Interface};
use crossbeam_channel::Receiver;
use jsonrpc_core::IoHandler;
use jsonrpc_ws_server::ServerBuilder;
use std::{net::SocketAddr, thread};
use tokio::runtime::Runtime;

pub struct WebsocketInterface {
    port: u16,
    bound_address: Option<SocketAddr>,
}

impl WebsocketInterface {
    pub fn new(port: u16) -> Self {
        WebsocketInterface {
            port,
            bound_address: None,
        }
    }

    pub fn bound_address(&self) -> Option<SocketAddr> {
        self.bound_address
    }
}

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CONDUCTOR_LIB)]
impl Interface for WebsocketInterface {
    fn run(
        &mut self,
        handler: IoHandler,
        kill_switch: Receiver<()>,
    ) -> Result<(Broadcaster, thread::JoinHandle<()>), String> {
        let url = format!("0.0.0.0:{}", self.port);
        let runtime = Runtime::new().map_err(|e| e.to_string())?;
        let server = ServerBuilder::new(handler)
            .event_loop_executor(runtime.executor())
            .start(&url.parse().expect("Invalid URL!"))
            .map_err(|e| e.to_string())?;
        self.bound_address = Some(*server.addr());
        let broadcaster = Broadcaster::Ws(server.broadcaster());
        let handle = thread::Builder::new()
            .name(format!("websocket_interface/{}", url))
            .spawn(move || {
                let _ = server; // move `server` into this thread
                let _ = runtime; // move tokio runtime for RPC futures into this thread
                let _ = kill_switch.recv();
            })
            .expect("Could not spawn thread for websocket interface");
        Ok((broadcaster, handle))
    }
}
