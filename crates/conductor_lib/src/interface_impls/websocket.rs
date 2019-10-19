use conductor::broadcaster::Broadcaster;
use crossbeam_channel::Receiver;
use interface::Interface;
use jsonrpc_core::IoHandler;
use jsonrpc_ws_server::ServerBuilder;
use std::thread;

pub struct WebsocketInterface {
    port: u16,
}

impl WebsocketInterface {
    pub fn new(port: u16) -> Self {
        WebsocketInterface { port }
    }
}

impl Interface for WebsocketInterface {
    fn run(
        &self,
        handler: IoHandler,
        kill_switch: Receiver<()>,
    ) -> Result<(Broadcaster, thread::JoinHandle<()>), String> {
        let url = format!("0.0.0.0:{}", self.port);
        let server = ServerBuilder::new(handler)
            .start(&url.parse().expect("Invalid URL!"))
            .map_err(|e| e.to_string())?;
        let broadcaster = Broadcaster::Ws(server.broadcaster());
        let handle = thread::Builder::new()
            .name(format!("websocket_interface/{}", url))
            .spawn(move || {
                let _ = server; // move `server` into this thread
                let _ = kill_switch.recv();
            })
            .expect("Could not spawn thread for websocket interface");
        Ok((broadcaster, handle))
    }
}
