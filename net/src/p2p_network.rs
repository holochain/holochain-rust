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

use super::{
    ipc_net_worker::IpcNetWorker,
    mock_worker::MockWorker,
    p2p_config::*,
};

use serde_json;


/// The p2p network instance
pub struct P2pNetwork {
    connection: NetConnectionThread,
    config: P2pConfig,
}

impl std::fmt::Debug for P2pNetwork {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "P2pNetwork {{}}")
    }
}

impl NetConnection for P2pNetwork {
    /// send a Protocol message to the p2p network instance
    fn send(&mut self, data: Protocol) -> NetResult<()> {
        self.connection.send(data)
    }
}

impl P2pNetwork {
    /// create a new p2p network instance, given message handler and config json
    pub fn new(handler: NetHandler, config_json: &JsonString) -> NetResult<Self> {
        // Create Config struct
        let config: serde_json::Value = serde_json::from_str(config_json.into())?;

        // so far, we have only implemented the "ipc" backend type
        let connection = match config["backend"].to_string().as_str() {
            "\"ipc\"" => {
                // create a new ipc backend with the passed sub "config" info
                NetConnectionThread::new(
                    handler,
                    Box::new(move |h| {
                        let out: Box<NetWorker> = Box::new(IpcNetWorker::new(
                            h,
                            &(config["config"].to_string().into()),
                        )?);
                        Ok(out)
                    }),
                    None,
                )?
            }
            "\"mock\"" => {
                 NetConnectionThread::new(
                    handler,
                    Box::new(move |h| Ok(Box::new(MockWorker::new(h)?) as Box<NetWorker>)),
                    None,
                )?
            },
            _ => bail!("unknown p2p_network backend: {}", config["backend"]),
        };
        Ok(P2pNetwork {
            connection,
            config: P2pConfig { backend_kind: P2pBackendKind::MOCK, backend_config: config_json.clone()},
    })
    }

    /// stop the network module (disconnect any sockets, join any threads, etc)
    pub fn stop(self) -> NetResult<()> {
        self.connection.stop()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_fail_bad_backend_type() {
        if let Err(e) = P2pNetwork::new(
            Box::new(|_r| Ok(())),
            &json!({
                "backend": "bad"
            })
            .to_string()
            .into(),
        ) {
            let e = format!("{:?}", e);
            assert!(e.contains("backend: \\\"bad\\\""), "res: {}", e);
        } else {
            panic!("should have thrown");
        }
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
            })
            .into(),
        )
        .unwrap();
        res.send(Protocol::P2pReady).unwrap();
        res.stop().unwrap();
    }

    #[test]
    fn it_should_create_mock() {
        let mut res = P2pNetwork::new(
            Box::new(|_r| Ok(())),
            &json!({
                "backend": "mock"
            })
            .into(),
        )
        .unwrap();
        res.send(Protocol::P2pReady).unwrap();
        res.stop().unwrap();
    }
}
