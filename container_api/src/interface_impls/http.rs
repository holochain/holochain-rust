use interface::Interface;
use jsonrpc_http_server::{jsonrpc_core::IoHandler, ServerBuilder};

pub struct HttpInterface {
    port: u16,
}

impl HttpInterface {
    pub fn new(port: u16) -> Self {
        HttpInterface { port }
    }
}

impl Interface for HttpInterface {
    fn run(&self, handler: IoHandler) -> Result<(), String> {
        let url = format!("0.0.0.0:{}", self.port);
        let server = ServerBuilder::new(handler)
            .start_http(&url.parse().expect("Invalid URL!"))
            .map_err(|e| e.to_string())?;
        server.wait();
        Ok(())
    }
}
