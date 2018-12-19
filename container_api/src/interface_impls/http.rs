// use holochain_core_types::json::JsonString;

use std::sync::Arc;
use tiny_http::{Response, Server};
use jsonrpc_ws_server::jsonrpc_core::{self, IoHandler, Value};
use interface::{DispatchRpc, Interface};

pub struct HttpInterface {
    port: u16,
}

/// TODO: this is a stub, to be implemented later
impl HttpInterface {
    pub fn new(port: u16) -> Self {
        Self { port }
    }
}

impl Interface for HttpInterface {
    fn run(&self, _handlerr: IoHandler) -> Result<(), String> {
        let server_url = format!("0.0.0.0:{}", self.port);
        let server = Server::http(server_url.as_str()).unwrap();
        for request in server.incoming_requests() {
            let method = request.url().to_string();
            println!("{}", method);
            let response = Response::from_string(method);
            request.respond(response).unwrap();
        }
        unimplemented!();
    }
}
