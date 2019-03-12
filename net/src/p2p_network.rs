//! This module provides the main abstraction for differing p2p backends
//! P2pNetwork instances take a json configuration string
//! and at load-time instantiate the configured "backend"

use crate::connection::{
    net_connection::{NetHandler, NetSend, NetWorker, NetWorkerFactory},
    net_connection_thread::NetConnectionThread,
    protocol::Protocol,
    NetResult,
};
use std::{thread::sleep, time::Duration};

use crate::{
    in_memory::memory_worker::InMemoryWorker, ipc_net_worker::IpcNetWorker, p2p_config::*,
};

/// Facade handling a p2p module responsable for the network connection
/// Holds a NetConnectionThread and implements itself the NetSend Trait
/// `send()` is used for sending Protocol messages to the network
/// `handler` closure provide on construction for handling Protocol messages received from the network.
pub struct P2pNetwork {
    connection: NetConnectionThread,
}

impl P2pNetwork {
    /// Constructor
    /// `config` is the configuration of the p2p module
    /// `handler` is the closure for handling Protocol messages received from the network.
    pub fn new(handler: NetHandler, p2p_config: &P2pConfig) -> NetResult<Self> {
        // Create Config struct
        let backend_config = p2p_config.backend_config.to_string().into();
        // Provide worker factory depending on backend kind
        let worker_factory: NetWorkerFactory = match p2p_config.backend_kind {
            // Create an IpcNetWorker with the passed backend config
            P2pBackendKind::IPC => {
                let enduser_config = p2p_config
                    .maybe_end_user_config
                    .clone()
                    .expect("P2pConfig for IPC networking is missing an end-user config")
                    .to_string();
                Box::new(move |h| {
                    Ok(
                        Box::new(IpcNetWorker::new(h, &backend_config, enduser_config)?)
                            as Box<NetWorker>,
                    )
                })
            }
            // Create an InMemoryWorker
            P2pBackendKind::MEMORY => Box::new(move |h| {
                Ok(Box::new(InMemoryWorker::new(h, &backend_config)?) as Box<NetWorker>)
            }),
        };
        // Create NetConnectionThread with appropriate worker factory
        let connection = NetConnectionThread::new(handler, worker_factory, None)?;
        if let P2pBackendKind::IPC = p2p_config.backend_kind {
            // TODO: fix this by handling initialization ordering properly
            sleep(Duration::from_millis(1000));
        }
        // Done
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

impl NetSend for P2pNetwork {
    /// send a Protocol message to the p2p network instance
    fn send(&mut self, data: Protocol) -> NetResult<()> {
        self.connection.send(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_memory_network() {
        let mut res = P2pNetwork::new(
            Box::new(|_r| Ok(())),
            &P2pConfig::new_with_unique_memory_backend(),
        )
        .unwrap();
        res.send(Protocol::P2pReady).unwrap();
        res.stop().unwrap();
    }
}
