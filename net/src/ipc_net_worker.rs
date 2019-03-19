//! provides a NetWorker implementation for backend IPC p2p connections

use holochain_core_types::json::JsonString;

use crate::ipc::{
    spawn, transport::TransportId, util::get_millis, Transport, TransportEvent, TransportWss,
};

use crate::connection::{
    json_protocol::{ConfigData, ConnectData, JsonProtocol, StateData},
    net_connection::{NetHandler, NetShutdown, NetWorker},
    protocol::{NamedBinaryData, Protocol},
    NetResult,
};

use std::{collections::HashMap, convert::TryFrom};

use crate::tweetlog::TweetProxy;

use serde_json;

/// a NetWorker talking to the network via another process through an IPC connection.
pub struct IpcNetWorker {
    /// Function that will forwarded the incoming network messages
    handler: NetHandler,
    wss_socket: TransportWss<std::net::TcpStream>,
    ipc_uri: String,
    transport_id: TransportId,
    done: NetShutdown,

    is_network_ready: bool,
    last_known_state: String,
    last_state_millis: f64,

    bootstrap_nodes: Vec<String>,

    log: TweetProxy,
}

/// Constructors
impl IpcNetWorker {
    /// Public Constructor with config as a json string
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
            return IpcNetWorker::priv_new(handler, uri.to_string(), None, bootstrap_nodes);
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
    fn priv_new_with_spawn(
        handler: NetHandler,
        work_dir: String,
        config: String,
        env: HashMap<String, String>,
        bootstrap_nodes: Vec<String>,
    ) -> NetResult<Self> {
        // Spawn a process with given `cmd` that we will have an IPC connection with
        let spawn_result = spawn::ipc_spawn(work_dir, config, env, true)?;
        // Get spawn result info
        let ipc_binding = spawn_result.ipc_binding;
        let kill = spawn_result.kill;
        // Done
        IpcNetWorker::priv_new(handler, ipc_binding, kill, bootstrap_nodes)
    }

    /// Constructor without config
    fn priv_new(
        handler: NetHandler,
        ipc_uri: String,
        done: NetShutdown,
        bootstrap_nodes: Vec<String>,
    ) -> NetResult<Self> {
        let log = TweetProxy::new("IpcNetWorker");
        log.i(&format!("connect to uri {}", ipc_uri));

        let mut wss_socket = TransportWss::with_std_tcp_stream();
        let transport_id = wss_socket.wait_connect(&ipc_uri)?;

        log.i(&format!("connection success. tId = {}", transport_id));

        Ok(IpcNetWorker {
            handler,
            wss_socket,
            ipc_uri,
            transport_id,
            done,
            is_network_ready: false,
            last_known_state: "undefined".to_string(),
            last_state_millis: 0.0_f64,
            bootstrap_nodes,
            log,
        })
    }
}

impl NetWorker for IpcNetWorker {
    /// stop the net worker
    fn stop(mut self: Box<Self>) -> NetResult<()> {
        self.wss_socket.close_all()?;
        if let Some(done) = self.done {
            done();
        }
        Ok(())
    }

    /// we got a message from holochain core
    /// (just forwards to the internal worker relay)
    fn receive(&mut self, data: Protocol) -> NetResult<()> {
        let data: NamedBinaryData = data.into();
        let data = rmp_serde::to_vec_named(&data)?;
        self.wss_socket.send_all(&data)?;

        Ok(())
    }

    /// do some upkeep on the internal worker
    /// IPC server state handling / magic
    fn tick(&mut self) -> NetResult<bool> {
        // Request p2p module's state if its not ready yet
        if &self.last_known_state != "ready" {
            self.priv_request_state()?;
        }

        let (did_work, evt_lst) = self.wss_socket.poll()?;
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
                    let msg: NamedBinaryData = rmp_serde::from_slice(&msg)?;
                    let msg: Protocol = msg.into();

                    // handle init/config special cases
                    if let Ok(msg) = JsonProtocol::try_from(&msg) {
                        match msg {
                            // ipc-server sent us its current state
                            JsonProtocol::GetStateResult(state) => {
                                self.priv_handle_state(state)?;
                            }
                            // ipc-server is requesting us the default config
                            JsonProtocol::GetDefaultConfigResult(config) => {
                                self.priv_handle_default_config(config)?;
                            }
                            _ => (),
                        };
                    }
                    // Send data back to handler
                    (self.handler)(Ok(msg))?;

                    // When p2p module is ready:
                    // - Notify handler that the p2p module is ready
                    // - Try connecting to boostrap nodes
                    if !self.is_network_ready && &self.last_known_state == "ready" {
                        self.is_network_ready = true;
                        (self.handler)(Ok(Protocol::P2pReady))?;
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
}

// private
impl IpcNetWorker {
    // Send 'Connect to bootstrap nodes' request to Ipc server
    fn priv_send_connects(&mut self) -> NetResult<()> {
        let bs_nodes: Vec<String> = self.bootstrap_nodes.drain(..).collect();
        for bs_node in &bs_nodes {
            self.receive(
                JsonProtocol::Connect(ConnectData {
                    peer_address: bs_node.clone().into(),
                })
                .into(),
            )?;
        }

        Ok(())
    }

    /// send a ping and/or? StateRequest twice per second
    fn priv_request_state(&mut self) -> NetResult<()> {
        let now = get_millis();

        if now - self.last_state_millis > 500.0 {
            self.receive(JsonProtocol::GetState.into())?;
            self.last_state_millis = now;
        }

        Ok(())
    }

    /// Handle State Message received from IPC server.
    fn priv_handle_state(&mut self, state: StateData) -> NetResult<()> {
        // Keep track of IPC server's state
        self.last_known_state = state.state;
        // if the internal worker needs configuration, fetch the default config
        if &self.last_known_state == "need_config" {
            self.receive(JsonProtocol::GetDefaultConfig.into())?;
        }
        Ok(())
    }

    /// Handle DefaultConfig Message received from ipc-server.
    /// Pass it back the default config only if it needs configurating
    fn priv_handle_default_config(&mut self, config_msg: ConfigData) -> NetResult<()> {
        if &self.last_known_state == "need_config" {
            self.receive(
                JsonProtocol::SetConfig(ConfigData {
                    config: config_msg.config,
                })
                .into(),
            )?;
        }

        Ok(())
    }
}
