use holochain_core_types::{error::HolochainError, json::JsonString};
use Holochain;

use jsonrpc::JsonRpc;
use serde_json::{self, Value};
use std::{
    collections::HashMap,
    convert::TryFrom,
    sync::{Arc, Mutex},
    thread,
};
use tiny_http::{Response, Server};

use interface::{DispatchRpc, Interface};

pub struct HttpInterface {
    port: u16,
}

impl HttpInterface {
    pub fn new(port: u16) -> Self {
        Self { port }
    }
}

impl Interface for HttpInterface {
    fn run(&self, _dispatcher: Arc<DispatchRpc>) -> Result<(), String> {
        unimplemented!();

        let server_url = format!("0.0.0.0:{}", self.port);
        let server = Server::http(server_url.as_str()).unwrap();
        for request in server.incoming_requests() {
            let method = request.url().to_string();
            println!("{}", method);
            let response = Response::from_string(method);
            request.respond(response).unwrap();
        }
        Ok(())
    }
}

fn mk_err(msg: &str) -> JsonString {
    json!({ "error": Value::from(msg) }).into()
}
