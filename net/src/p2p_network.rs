//! This module provides the main abstraction for differing p2p backends
//! P2pNetwork instances take a json configuration string
//! and at load-time instantiate the configured "backend"

use holochain_net_connection::{
    net_connection::{NetHandler, NetSend, NetWorker, NetWorkerFactory},
    net_connection_thread::NetConnectionThread,
    protocol::Protocol,
    NetResult,
};

use super::{ipc_net_worker::IpcNetWorker, memory_worker::InMemoryWorker, p2p_config::*};

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
    pub fn new(handler: NetHandler, config: &P2pConfig) -> NetResult<Self> {
        println!("config = {:?}", config);
        // Create Config struct
        let network_config = config.backend_config.to_string().into();
        // Provide worker factory depending on backend kind
        let worker_factory: NetWorkerFactory = match config.backend_kind {
            // Create an IpcNetWorker with the passed backend config
            P2pBackendKind::IPC => Box::new(move |h| {
                Ok(Box::new(IpcNetWorker::new(h, &network_config)?) as Box<NetWorker>)
            }),
            // Create an InMemoryWorker
            P2pBackendKind::MEMORY => Box::new(move |h| {
                Ok(Box::new(InMemoryWorker::new(h, &network_config)?) as Box<NetWorker>)
            }),
        };
        // Create NetConnectionThread with appropriate worker factory
        let connection = NetConnectionThread::new(handler, worker_factory, None)?;
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
