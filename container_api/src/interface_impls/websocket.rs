use jsonrpc_ws_server::ServerBuilder;

use interface::{ContainerApiDispatcher, DispatchRpc, Interface};

pub struct WebsocketInterface {
    port: u16,
}

impl WebsocketInterface {
    pub fn new(port: u16) -> Self {
        WebsocketInterface { port }
    }
}

impl Interface<ContainerApiDispatcher> for WebsocketInterface {
    fn run(&self, dispatcher: ContainerApiDispatcher) -> Result<(), String> {
        let io = dispatcher.handler();
        let url = format!("0.0.0.0:{}", self.port);
        let server = ServerBuilder::new(io)
            .start(&url.parse().expect("Invalid URL!"))
            .map_err(|e| e.to_string())?;
        server.wait().map_err(|e| e.to_string())?;
        Ok(())
    }
}
