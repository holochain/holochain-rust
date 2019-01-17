use interface::Interface;
use jsonrpc_ws_server::{jsonrpc_core::IoHandler, ServerBuilder};
use std::sync::mpsc::Receiver;

pub struct WebsocketInterface {
    port: u16,
}

impl WebsocketInterface {
    pub fn new(port: u16) -> Self {
        WebsocketInterface { port }
    }
}

impl Interface for WebsocketInterface {
    fn run(&self, handler: IoHandler, kill_switch: Receiver<()>) -> Result<(), String> {
        let url = format!("0.0.0.0:{}", self.port);
        let _server = ServerBuilder::new(handler)
            .start(&url.parse().expect("Invalid URL!"))
            .map_err(|e| e.to_string())?;
        let _ = kill_switch.recv();
        Ok(())
    }
}
