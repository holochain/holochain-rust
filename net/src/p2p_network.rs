//! This module provides the core abstraction for differing p2p backends
//! P2pNetwork instances take a json configuration string
//! and at load-time instantiate the configured "backend"

use holochain_core_types::json::JsonString;

use holochain_net_connection::{
    net_connection::{NetConnection, NetHandler, NetWorker},
    net_connection_thread::NetConnectionThread,
    protocol::Protocol,
    NetResult,
};

use super::ipc_net_worker::IpcNetWorker;

use serde_json;

/// The p2p network instance
#[derive(Debug)]
pub struct P2pNetwork {
    con: NetConnectionThread,
}

impl NetConnection for P2pNetwork {
    /// send a Protocol message to the p2p network instance
    fn send(&mut self, data: Protocol) -> NetResult<()> {
        self.con.send(data)
    }
}

impl P2pNetwork {
    /// create a new p2p network instance, given message handler and config json
    pub fn new(handler: NetHandler, config: &JsonString) -> NetResult<Self> {
        let config: serde_json::Value = serde_json::from_str(config.into())?;

        // so far, we have only implemented the "ipc" backend type
        if &config["backend"] == "ipc" {
            // create a new ipc backend with the passed sub "config" info
            return Ok(P2pNetwork {
                con: NetConnectionThread::new(
                    handler,
                    Box::new(move |h| {
                        let out: Box<NetWorker> = Box::new(IpcNetWorker::new(
                            h,
                            &(config["config"].to_string().into()),
                        )?);
                        Ok(out)
                    }),
                )?,
            });
        }

        bail!("unknown p2p_network backend: {}", config["backend"]);
    }

    /// stop the network module (disconnect any sockets, join any threads, etc)
    pub fn stop(self) -> NetResult<()> {
        self.con.stop()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_fail_bad_backend_type() {
        let res = P2pNetwork::new(
            Box::new(|_r| Ok(())),
            &json!({
                "backend": "bad"
            }).to_string()
                .into(),
        ).expect_err("should have thrown")
            .to_string();
        assert!(res.contains("backend: \"bad\""), "res: {}", res);
    }

    #[test]
    fn it_should_create_zmq_socket() {
        let mut res = P2pNetwork::new(
            Box::new(|_r| Ok(())),
            &json!({
                "backend": "ipc",
                "config": {
                    "socketType": "zmq",
                    "ipcUri": "tcp://127.0.0.1:0",
                    "blockConnect": false
                }
            }).into(),
        ).unwrap();
        res.send(Protocol::P2pReady).unwrap();
        res.stop().unwrap();
    }
}
