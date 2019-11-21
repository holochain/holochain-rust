//! provides a NetWorker implementation for backend IPC p2p connections

use holochain_json_api::json::JsonString;

use crate::ipc::{spawn, transport::TransportId, Transport, TransportEvent, TransportWss};

use crate::connection::{
    net_connection::{NetHandler, NetShutdown, NetWorker},
    NetResult,
};

use lib3h_protocol::{
    data_types::ConnectData, protocol_client::Lib3hClientProtocol,
    protocol_server::Lib3hServerProtocol, types::NetworkHash,
};

use std::collections::HashMap;

use crate::tweetlog::TweetProxy;

use serde_json;

/// a NetWorker talking to the network via another process through an IPC connection.
#[allow(dead_code)] // for handler which is temporarily disabled
pub struct IpcNetWorker {
    /// Function that will forwarded the incoming network messages
    handler: NetHandler,
    wss_socket: TransportWss<std::net::TcpStream>,
    ipc_uri: String,
    p2p_uri: String,
    transport_id: TransportId,
    done: NetShutdown,

    is_network_ready: bool,
    last_known_state: String,

    bootstrap_nodes: Vec<String>,

    log: TweetProxy,
}

/// Constructors
impl IpcNetWorker {
    /// Public Constructor with config as a json string
    #[cfg(not(target_arch = "wasm32"))]
    #[flame]
    pub fn new(
        handler: NetHandler,
        config: &JsonString,
        enduser_config: String,
    ) -> NetResult<Self> {
        // Load config
        let config: serde_json::Value = serde_json::from_str(config.into())?;
        let empty = vec![];
        let bootstrap_nodes: Vec<String> = config["bootstrapNodes"]
            .as_array()
            .unwrap_or(&empty)
            .iter()
            .map(|s| s.as_str().unwrap().to_string())
            .collect();
        // Create a new IpcNetWorker that connects to the ptovided 'ipcUri'
        if let Some(uri) = config["ipcUri"].as_str() {
            return IpcNetWorker::priv_new(handler, uri.to_string(), None, None, bootstrap_nodes);
        }
        // No 'ipcUri' provided in config so use 'spawn' config instead
        // Check 'spawn' config
        if config["spawn"].as_object().is_none() {
            bail!("config.spawn or config.ipcUri is required");
        }
        let spawn_config = config["spawn"].as_object().unwrap();
        if !(spawn_config["workDir"].is_string() && spawn_config["env"].is_object()) {
            bail!("config.spawn requires 'workDir', and 'env'");
        }
        let env: HashMap<String, String> = spawn_config["env"]
            .as_object()
            .unwrap()
            .iter()
            .map(|(k, v)| (k.to_string(), v.as_str().unwrap().to_string()))
            .collect();
        // create a new IpcNetWorker witch spawns the n3h process
        return IpcNetWorker::priv_new_with_spawn(
            handler,
            spawn_config["workDir"].as_str().unwrap().to_string(),
            enduser_config,
            env,
            bootstrap_nodes,
        );
    }

    /// Constructor with IpcNetWorker instance pointing to a process that we spawn here
    #[cfg(not(target_arch = "wasm32"))]
    #[flame]
    fn priv_new_with_spawn(
        handler: NetHandler,
        work_dir: String,
        config: String,
        env: HashMap<String, String>,
        bootstrap_nodes: Vec<String>,
    ) -> NetResult<Self> {
        // Spawn a process with given `cmd` that we will have an IPC connection with
        let spawn_result =
            spawn::ipc_spawn(work_dir, config, env, spawn::DEFAULT_TIMEOUT_MS, true)?;
        // Get spawn result info
        let ipc_binding = spawn_result.ipc_binding;
        let kill = spawn_result.kill;
        // Done
        IpcNetWorker::priv_new(
            handler,
            ipc_binding,
            Some(spawn_result.p2p_bindings[0].clone()),
            kill,
            bootstrap_nodes,
        )
    }

    /// Constructor without config
    #[cfg(not(target_arch = "wasm32"))]
    #[flame]
    fn priv_new(
        handler: NetHandler,
        ipc_uri: String,
        p2p_uri: Option<String>,
        done: NetShutdown,
        bootstrap_nodes: Vec<String>,
    ) -> NetResult<Self> {
        let log = TweetProxy::new("IpcNetWorker");
        log.i(&format!("connect to uri {}", ipc_uri));

        let mut wss_socket = TransportWss::with_std_tcp_stream();
        let transport_id = wss_socket.wait_connect(&ipc_uri)?;

        log.i(&format!("connection success. ipc tId = {}", transport_id));

        Ok(IpcNetWorker {
            handler,
            wss_socket,
            ipc_uri,
            p2p_uri: match p2p_uri {
                Some(p2p_uri) => p2p_uri,
                None => String::new(),
            },
            transport_id,
            done,
            is_network_ready: false,
            last_known_state: "undefined".to_string(),
            bootstrap_nodes,
            log,
        })
    }
}

impl NetWorker for IpcNetWorker {
    /// stop the net worker
    #[cfg(not(target_arch = "wasm32"))]
    #[flame]
    fn stop(mut self: Box<Self>) -> NetResult<()> {
        // Nothing to do if sub-process already terminated
        if self.last_known_state == "terminated" {
            return Ok(());
        }
        let _ = self.tick();
        // Close connection and kill process
        self.wss_socket.close_all()?;
        if let Some(mut done) = self.done {
            done();
        }
        // Done
        Ok(())
    }

    /// we got a message from holochain core
    /// (just forwards to the internal worker relay)
    #[cfg(not(target_arch = "wasm32"))]
    #[flame]
    fn receive(&mut self, data: Lib3hClientProtocol) -> NetResult<()> {
        let data = serde_json::to_string_pretty(&data)?;
        self.wss_socket.send_all(data.as_bytes())?;
        Ok(())
    }

    /// do some upkeep on the internal worker
    /// IPC server state handling / magic
    #[cfg(not(target_arch = "wasm32"))]
    #[flame]
    fn tick(&mut self) -> NetResult<bool> {
        let (did_work, evt_lst) = self.wss_socket.poll()?;
        if evt_lst.len() > 0 {
            self.last_known_state = "ready".to_string();
        }
        //println!("@@@tick");
        for evt in evt_lst {
            match evt {
                TransportEvent::TransportError(_id, e) => {
                    self.log.e(&format!("ipc ws error {:?}", e));
                    self.wss_socket.close(self.transport_id.clone())?;
                    self.transport_id = self.wss_socket.wait_connect(&self.ipc_uri)?;
                }
                TransportEvent::Connect(_id) => {
                    // don't need to do anything here
                }
                TransportEvent::Close(_id) => {
                    self.log.e("ipc ws closed");
                    self.wss_socket.close(self.transport_id.clone())?;
                    self.transport_id = self.wss_socket.wait_connect(&self.ipc_uri)?;
                }
                TransportEvent::Message(_id, msg) => {
                    let msg: Lib3hServerProtocol = serde_json::from_slice(&msg)?;
                    self.handler.handle(Ok(msg.clone()))?;

                    // on shutdown, close all connections
                    if msg == Lib3hServerProtocol::Terminated {
                        self.is_network_ready = false;
                        self.last_known_state = "terminated".to_string();
                        let res = self.wss_socket.close_all();
                        if let Err(e) = res {
                            self.log.w(&format!("Error while stopping worker: {:?}", e));
                        }
                    }
                    // When p2p module is ready:
                    // - Notify handler that the p2p module is ready
                    // - Try connecting to boostrap nodes
                    if !self.is_network_ready && &self.last_known_state == "ready" {
                        self.is_network_ready = true;
                        self.handler.handle(Ok(Lib3hServerProtocol::P2pReady))?;
                        self.priv_send_connects()?;
                    }
                }
            }
        }

        Ok(did_work)
    }

    /// Getter
    fn endpoint(&self) -> Option<String> {
        Some(self.ipc_uri.clone())
    }

    fn p2p_endpoint(&self) -> Option<url::Url> {
        match url::Url::parse(&self.p2p_uri) {
            Err(_) => None,
            Ok(u) => Some(u),
        }
    }
}

// private
impl IpcNetWorker {
    // Send 'Connect to bootstrap nodes' request to Ipc server
    fn priv_send_connects(&mut self) -> NetResult<()> {
        let bs_nodes: Vec<String> = self.bootstrap_nodes.drain(..).collect();
        for bs_node in &bs_nodes {
            let uri = match url::Url::parse(bs_node.as_str()) {
                Ok(uri) => uri,
                Err(e) => {
                    self.log.w(&format!("{:?}: {:?}", e, bs_node.as_str()));
                    continue;
                }
            };
            self.receive(Lib3hClientProtocol::Connect(ConnectData {
                request_id: snowflake::ProcessUniqueId::new().to_string(),
                peer_location: uri.into(),
                network_id: NetworkHash::default(),
            }))?
        }

        Ok(())
    }
}
