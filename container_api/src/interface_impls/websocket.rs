use interface::Interface;
use jsonrpc_ws_server::{jsonrpc_core::IoHandler, ServerBuilder};

pub struct WebsocketInterface {
    port: u16,
}

impl WebsocketInterface {
    pub fn new(port: u16) -> Self {
        WebsocketInterface { port }
    }
}

impl Interface for WebsocketInterface {
    fn run(&self, handler: IoHandler) -> Result<(), String> {
        let url = format!("0.0.0.0:{}", self.port);
        let server = ServerBuilder::new(handler)
            .start(&url.parse().expect("Invalid URL!"))
            .map_err(|e| e.to_string())?;
        server.wait().map_err(|e| e.to_string())?;
        Ok(())
    }
}
