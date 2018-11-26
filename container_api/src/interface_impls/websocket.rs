use holochain_core_types::json::JsonString;

use jsonrpc::JsonRpcRequest;
use jsonrpc_ws_server::{
    jsonrpc_core::{middleware, IoHandler, MetaIoHandler},
    ServerBuilder,
};
use serde_json::Value;
use std::{convert::TryFrom, sync::Arc};
use ws::{self, Message};

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
