use holochain_core_types::json::JsonString;

use jsonrpc::JsonRpcRequest;
use serde_json::Value;
use std::{convert::TryFrom, sync::Arc};
use ws::{self, Message};
use jsonrpc_ws_server::{
    ServerBuilder,
    jsonrpc_core::{MetaIoHandler, IoHandler, middleware}
};

use interface::{DispatchRpc, Interface, ContainerApiDispatcher};

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
        // let mut io = IoHandler::new();
        let io = dispatcher.handler();
        // let io: MetaIoHandler<(), middleware::Noop> = dispatcher.handler();
        let url = format!("0.0.0.0:{}", self.port);
        let server = ServerBuilder::new(io)
            .start(&url.parse().unwrap()).unwrap();
        server.wait().unwrap();
        Ok(())
    }
}

