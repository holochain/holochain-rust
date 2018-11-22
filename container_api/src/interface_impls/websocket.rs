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
use ws::{self, Message, Result as WsResult};

use interface::{DispatchRpc, Interface};

pub struct WebsocketInterface {
    port: u16,
}

impl WebsocketInterface {
    pub fn new(port: u16) -> Self {
        WebsocketInterface { port }
    }
}

impl Interface for WebsocketInterface {
    fn run(&self, dispatcher: Arc<DispatchRpc>) -> Result<(), String> {
        ws::listen(format!("localhost:{}", self.port), move |out| {
            // must clone the Arc as we move from outer FnMut to inner FnMut
            let dispatcher = dispatcher.clone();
            move |msg| match msg {
                Message::Text(s) => match JsonRpc::try_from(s) {
                    Ok(ref rpc) => {
                        let response: JsonString = match dispatcher.dispatch_rpc(rpc) {
                            Ok(payload) => payload.clone(),
                            Err(err) => mk_err(&err.to_string()).clone(),
                        };
                        out.send(Message::Text(response.into()))
                    }
                    Err(err) => out.send(Message::Text(mk_err(&err).to_string())),
                },
                Message::Binary(_b) => unimplemented!(),
            }
        }).map_err(|e| e.to_string())
    }
}

fn mk_err(msg: &str) -> JsonString {
    json!({ "error": Value::from(msg) }).into()
}
