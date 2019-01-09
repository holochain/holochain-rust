//! This module provides the main abstraction for differing p2p backends
//! P2pNetwork instances take a json configuration string
//! and at load-time instantiate the configured "backend"

use holochain_net_connection::{
    net_connection::{NetConnection, NetHandler, NetWorker},
    net_connection_thread::NetConnectionThread,
    protocol::Protocol,
    NetResult,
};

use super::{ipc_net_worker::IpcNetWorker, mock_worker::MockWorker, p2p_config::*};

/// Facade handling a network connection
/// Holds a NetConnectionThread and implements itself the NetConnection Trait
pub struct P2pNetwork {
    connection: NetConnectionThread,
}

impl P2pNetwork {
    /// Create a new p2p network connection
    /// `config` is the configuration of the p2p connection
    /// `handler` is the closure for handling received Protocol messages
    /// `send()` is used for sending Protocol messages to the network
    pub fn new(handler: NetHandler, config: &P2pConfig) -> NetResult<Self> {
        // Create Config struct
        let network_config = config.backend_config.to_string().into();
        // so far, we have only implemented the "ipc" backend type
        let connection = match config.backend_kind {
            P2pBackendKind::IPC => {
                // create a new ipc backend with the passed sub "config" info
                NetConnectionThread::new(
                    handler,
                    Box::new(move |h| {
                        let out: Box<NetWorker> = Box::new(IpcNetWorker::new(h, &network_config)?);
                        Ok(out)
                    }),
                    None,
                )?
            }
            P2pBackendKind::MOCK => NetConnectionThread::new(
                handler,
                Box::new(move |h| {
                    Ok(Box::new(MockWorker::new(h, &network_config)?) as Box<NetWorker>)
                }),
                None,
            )?,
        };
        Ok(P2pNetwork { connection })
    }

    /// Stop the network connection (disconnect any sockets, join any threads, etc)
    pub fn stop(self) -> NetResult<()> {
        self.connection.stop()
    }

    /// Getter of the endpoint of its connection
    pub fn endpoint(&self) -> String {
        self.connection.endpoint.clone()
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_zmq_socket() {
        let p2p_config = P2pConfig::new(
            P2pBackendKind::IPC,
            crate::ipc_net_worker::IpcNetWorker::ZMQ_URI_CONFIG,
        );
        let mut res = P2pNetwork::new(Box::new(|_r| Ok(())), &p2p_config).unwrap();
        res.send(Protocol::P2pReady).unwrap();
        res.stop().unwrap();
    }

    #[test]
    fn it_should_create_mock() {
        let mut res = P2pNetwork::new(Box::new(|_r| Ok(())), &P2pConfig::unique_mock()).unwrap();
        res.send(Protocol::P2pReady).unwrap();
        res.stop().unwrap();
    }
}
