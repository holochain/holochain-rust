use Holochain;
use holochain_core_types::{
    error::HolochainError, json::JsonString,
};

use std::{
    collections::HashMap,
    convert::TryFrom,
    sync::{Arc, Mutex, RwLock},
    thread,
};
use jsonrpc::JsonRpc;
use serde_json::{self, Value};
use ws::{self, Message, Result as WsResult};

pub type InterfaceError = String;
pub type InstanceMap = HashMap<String, Mutex<Holochain>>;


pub trait DispatchRpc {

    fn instances(&self) -> &InstanceMap;

    /// Dispatch to the correct Holochain and `call` it based on the JSONRPC method
    fn dispatch_rpc(&self, rpc: JsonRpc) -> Result<JsonString, HolochainError> {
        let matches: Vec<&str> = rpc.method.split('/').collect();
        let result = if let [instance_id, zome, cap, func] = matches.as_slice() {
            let key = instance_id.to_string();
            self.instances()
                .get(&key)
                .ok_or(format!("No instance with ID: {:?}", key))
                .and_then(|hc_mutex| {
                    let mut hc = hc_mutex.lock().unwrap();
                    hc.call(zome, cap, func, &rpc.params.to_string()).map_err(|e| e.to_string())
                })
        } else {
            Err(format!("bad rpc method: {}", rpc.method))
        };
        result.map_err(HolochainError::ErrorGeneric)
    }
}

pub trait Interface<D: DispatchRpc> {
    fn run(&self, &D) -> Result<(), String>;
    //
    // fn start(&self) -> Result<(), InterfaceError> {
    //     self.handle = thread::spawn(move || {
    //         self.run()
    //     })
    // }
    //
    // fn stop(&self) -> Result<(), InterfaceError> {
    //     self.handle.join()
    // }

}

struct WebsocketInterface {
    port: u16
}

impl<D: DispatchRpc> Interface<D> for WebsocketInterface {
    fn run(&self, dispatcher: &D) -> Result<(), String> {
        ws::listen(format!("localhost:{}", self.port), |out| {
            move |msg| match msg {
                Message::Text(s) => match JsonRpc::try_from(s) {
                    Ok(rpc) => {
                        let response: JsonString =
                            match dispatcher.dispatch_rpc(rpc) {
                                Ok(payload) => payload,
                                Err(err) => mk_err(&err.to_string()),
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
