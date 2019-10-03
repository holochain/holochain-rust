//! This module provides the main abstraction for differing p2p backends
//! P2pNetwork instances take a json configuration string
//! and at load-time instantiate the configured "backend"

use crate::{
    connection::{
        net_connection::{NetHandler, NetSend, NetWorker, NetWorkerFactory},
        net_connection_thread::NetConnectionThread,
        NetResult,
    },
    ipc_net_worker::IpcNetWorker,
    lib3h_worker::Lib3hWorker,
    p2p_config::*,
    tweetlog::*,
};
use lib3h_protocol::{protocol_client::Lib3hClientProtocol, protocol_server::Lib3hServerProtocol};

use crossbeam_channel;
use holochain_json_api::json::JsonString;
use std::{convert::TryFrom, time::Duration};

const P2P_READY_TIMEOUT_MS: u64 = 5000;

/// Facade handling a p2p module responsable for the network connection
/// Holds a NetConnectionThread and implements itself the NetSend Trait
/// `send()` is used for sending Protocol messages to the network
/// `handler` closure provide on construction for handling Protocol messages received from the network.
pub struct P2pNetwork {
    connection: NetConnectionThread,
}

impl P2pNetwork {
    /// Constructor
    /// `config` is the configuration of the p2p module `handler` is the closure for handling Protocol messages received from the network module.
    pub fn new(mut handler: NetHandler, p2p_config: P2pConfig) -> NetResult<Self> {
        // Create Config struct
        let backend_config_str = match &p2p_config.backend_config {
            BackendConfig::Json(ref json) => JsonString::from_json(&json.to_string()),
            _ => JsonString::from(""),
        };

        let p2p_config_str = p2p_config.clone().as_str();
        let p2p_config2 = p2p_config.clone();

        // Provide worker factory depending on backend kind
        let worker_factory: NetWorkerFactory = match &p2p_config.clone().backend_kind {
            // Create an IpcNetWorker with the passed backend config
            P2pBackendKind::N3H => {
                let enduser_config = p2p_config
                    .maybe_end_user_config
                    .clone()
                    .expect("P2pConfig for N3H networking is missing an end-user config")
                    .to_string();
                Box::new(move |h| {
                    Ok(Box::new(IpcNetWorker::new(
                        h,
                        &backend_config_str,
                        enduser_config.clone(),
                    )?) as Box<dyn NetWorker>)
                })
            }
            // Create a Lib3hWorker
            P2pBackendKind::LIB3H => {
                let backend_config = match &p2p_config.clone().backend_config {
                    BackendConfig::Lib3h(config) => config.clone(),
                    _ => return Err(format_err!("mismatch backend type, expecting lib3h")),
                };

                Box::new(move |h| {
                    Ok(Box::new(Lib3hWorker::with_wss_transport(h, backend_config.clone())?)
                        as Box<dyn NetWorker>)
                })
            }
            // Create an InMemoryWorker
            P2pBackendKind::MEMORY => Box::new(move |h| {
                let backend_config = match &p2p_config.clone().backend_config {
                    BackendConfig::Memory(config) => config.clone(),
                    _ => return Err(format_err!("mismatch backend type, expecting memory")),
                };
                Ok(Box::new(Lib3hWorker::with_memory_transport(h, backend_config.clone())?)
                   as Box<dyn NetWorker>)
            }),
        };

        let (t, rx) = crossbeam_channel::unbounded();
        let tx = t.clone();
        let wrapped_handler = if Self::should_wait_for_p2p_ready(&p2p_config2.clone()) {
            NetHandler::new(Box::new(move |message| {
                let unwrapped = message.unwrap();
                let message = unwrapped.clone();
                match Lib3hServerProtocol::try_from(unwrapped.clone()) {
                    Ok(Lib3hServerProtocol::P2pReady) => {
                        tx.send(Lib3hServerProtocol::P2pReady).ok();
                        log_d!("net/p2p_network: sent P2pReady event")
                    }
                    Ok(_msg) => {}
                    Err(_protocol_error) => {
                        // TODO why can't I use the above variable?
                        // Generates compiler error.
                    }
                };
                handler.handle(Ok(message))
            }))
        } else {
            handler
        };

        // Create NetConnectionThread with appropriate worker factory.  Indicate *what*
        // configuration failed to produce a connection.
        let connection =
            NetConnectionThread::new(wrapped_handler, worker_factory, None).map_err(|e| {
                format_err!(
                    "Failed to obtain a connection to a p2p network module w/ config: {}: {} ",
                    p2p_config_str,
                    e
                )
            })?;
        if Self::should_wait_for_p2p_ready(&p2p_config2.clone()) {
            P2pNetwork::wait_p2p_ready(&rx);
        }

        // Done
        Ok(P2pNetwork { connection })
    }

    fn should_wait_for_p2p_ready(p2p_config: &P2pConfig) -> bool {
        match p2p_config.backend_kind {
            P2pBackendKind::LIB3H |
            P2pBackendKind::MEMORY => false,
            P2pBackendKind::N3H => true
        }
    }

    fn wait_p2p_ready(rx: &crossbeam_channel::Receiver<Lib3hServerProtocol>) {
        let maybe_message = rx.recv_timeout(Duration::from_millis(P2P_READY_TIMEOUT_MS));
        match maybe_message {
            Ok(Lib3hServerProtocol::P2pReady) => log_d!("net/p2p_network: received P2pReady event"),
            Ok(msg) => {
                log_d!("net/p2p_network: received unexpected event: {:?}", msg);
            }
            Err(e) => {
                log_e!("net/p2p_network: did not receive P2pReady: {:?}", e);
                panic!(
                    "p2p network not ready within alloted time of {:?} ms",
                    P2P_READY_TIMEOUT_MS
                );
            }
        };
    }

    /// Stop the network connection (disconnect any sockets, join any threads, etc)
    pub fn stop(self) -> NetResult<()> {
        self.connection.stop()
    }

    /// Getter of the endpoint of its connection
    pub fn endpoint(&self) -> String {
        self.connection.endpoint.clone()
    }

    pub fn p2p_endpoint(&self) -> url::Url {
        self.connection.p2p_endpoint.clone()
    }
}

impl std::fmt::Debug for P2pNetwork {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "P2pNetwork {{}}")
    }
}

impl NetSend for P2pNetwork {
    /// send a Protocol message to the p2p network instance
    fn send(&mut self, data: Lib3hClientProtocol) -> NetResult<()> {
        self.connection.send(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lib3h_protocol::{data_types::ConnectData,uri::Lib3hUri};

    #[test]
    fn it_should_create_memory_network() {
        let p2p = P2pConfig::new_with_unique_memory_backend();
        let handler = NetHandler::new(Box::new(|_r| Ok(())));
        let mut res = P2pNetwork::new(handler.clone(), p2p).unwrap();
        let connect_data = ConnectData {
            request_id: "memory_network_req_id".into(),
            peer_location: Lib3hUri(url::Url::parse("mem://test".into()).expect("well formed memory network url")),
            network_id: "test_net_id".into(),
        };

        handler
            .to_owned()
            .handle(Ok(Lib3hServerProtocol::P2pReady))
            .unwrap();
        res.send(Lib3hClientProtocol::Connect(connect_data))
            .unwrap();
        res.stop().unwrap();
    }
}
