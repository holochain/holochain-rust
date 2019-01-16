//! provides a NetWorker implementation for backend IPC p2p connections

use holochain_core_types::json::JsonString;

use holochain_net_ipc::{
    ipc_client::IpcClient,
    socket::{IpcSocket, MockIpcSocket, TestStruct, ZmqIpcSocket},
    spawn,
    util::get_millis,
};

use holochain_net_connection::{
    net_connection::{NetHandler, NetSend, NetShutdown, NetWorker, NetWorkerFactory},
    net_relay::NetConnectionRelay,
    protocol::Protocol,
    protocol_wrapper::{ConfigData, ConnectData, ProtocolWrapper, StateData},
    NetResult,
};

use std::{collections::HashMap, convert::TryFrom, sync::mpsc};

use serde_json;

/// a NetWorker talking to the network via another process through an IPC connection.
pub struct IpcNetWorker {
    handler: NetHandler,

    ipc_relay: NetConnectionRelay,
    ipc_relay_receiver: mpsc::Receiver<Protocol>,

    is_ready: bool,

    last_known_state: String,
    last_state_millis: f64,

    bootstrap_nodes: Vec<String>,
    endpoint: String,
}

// Constructors
impl IpcNetWorker {
    // Constructor with config as a json string
    pub fn new(handler: NetHandler, config: &JsonString) -> NetResult<Self> {
        // Load config
        let config: serde_json::Value = serde_json::from_str(config.into())?;
        // Only zmq protocol is handled for now
        if config["socketType"] != "zmq" {
            bail!("unexpected socketType: {}", config["socketType"]);
        }
        let block_connect = config["blockConnect"].as_bool().unwrap_or(true);
        let empty = vec![];
        let bootstrap_nodes: Vec<String> = config["bootstrapNodes"]
            .as_array()
            .unwrap_or(&empty)
            .iter()
            .map(|s| s.as_str().unwrap().to_string())
            .collect();
        if config["ipcUri"].as_str().is_none() {
            // No 'ipcUri' config so use 'spawn' config
            if config["spawn"].as_object().is_none() {
                bail!("config.spawn or ipcUri is required");
            }
            let spawn_config = config["spawn"].as_object().unwrap();
            if !(spawn_config["cmd"].is_string()
                && spawn_config["args"].is_array()
                && spawn_config["workDir"].is_string()
                && spawn_config["env"].is_object())
            {
                bail!("config.spawn requires 'cmd', 'args', 'workDir', and 'env'");
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
                spawn_config["cmd"].as_str().unwrap().to_string(),
                spawn_config["args"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|i| i.as_str().unwrap_or_default().to_string())
                    .collect(),
                spawn_config["workDir"].as_str().unwrap().to_string(),
                env,
                block_connect,
                bootstrap_nodes,
            );
        }
        // create a new IpcNetWorker that connects to the given 'ipcUri'
        let uri = config["ipcUri"].as_str().unwrap().to_string();
        let endpoint = uri.clone();
        IpcNetWorker::priv_new(
            handler,
            Box::new(move |h| {
                let mut socket = ZmqIpcSocket::new()?;
                socket.connect(&uri)?;
                let out: Box<NetWorker> = Box::new(IpcClient::new(h, socket, block_connect)?);
                Ok(out)
            }),
            None,
            bootstrap_nodes,
            endpoint,
        )
    }

    /// Constructor with MockIpcSocket on local network
    pub fn new_test(handler: NetHandler, test_struct: TestStruct) -> NetResult<Self> {
        let default_local_endpoint = "tcp://127.0.0.1:0";
        IpcNetWorker::priv_new(
            handler,
            Box::new(move |h| {
                let mut socket = MockIpcSocket::new_test(test_struct)?;
                socket.connect(default_local_endpoint)?;
                let out: Box<NetWorker> = Box::new(IpcClient::new(h, socket, true)?);
                Ok(out)
            }),
            None,
            vec![],
            default_local_endpoint.to_string(),
        )
    }
}

/// Private Constructors
impl IpcNetWorker {
    /// Constructor with IpcNetWorker instance pointing to a process that we spawn here
    fn priv_new_with_spawn(
        handler: NetHandler,
        cmd: String,
        args: Vec<String>,
        work_dir: String,
        env: HashMap<String, String>,
        block_connect: bool,
        bootstrap_nodes: Vec<String>,
    ) -> NetResult<Self> {
        // Spawn a process with given `cmd` that we will have an IPC connection with
        let spawn_result = spawn::ipc_spawn(cmd, args, work_dir, env, block_connect)?;
        // Get spawn result info
        let ipc_binding = spawn_result.ipc_binding;
        let kill = spawn_result.kill;
        let endpoint = ipc_binding.clone();

        // Create factory: Creates a Zmq IPC socket and an IpcClient NetWorker which uses it.
        let factory = Box::new(move |h| {
            let mut socket = ZmqIpcSocket::new()?;
            socket.connect(&ipc_binding)?;
            let out: Box<NetWorker> = Box::new(IpcClient::new(h, socket, block_connect)?);
            Ok(out)
        });

        // Done
        IpcNetWorker::priv_new(handler, factory, kill, bootstrap_nodes, endpoint)
    }

    /// Constructor without config
    /// Using a NetConnectionRelay as socket
    fn priv_new(
        handler: NetHandler,
        factory: NetWorkerFactory,
        done: NetShutdown,
        bootstrap_nodes: Vec<String>,
        endpoint: String,
    ) -> NetResult<Self> {
        let (ipc_relay_sender, ipc_relay_receiver) = mpsc::channel::<Protocol>();

        let ipc_relay = NetConnectionRelay::new(
            Box::new(move |data| {
                // Relay valid data received from its worker (the network) back to its receiver (IpcNetWorker)
                ipc_relay_sender.send(data?)?;
                Ok(())
            }),
            factory,
            done,
        )?;

        Ok(IpcNetWorker {
            handler,
            ipc_relay,
            ipc_relay_receiver,
            is_ready: false,
            last_known_state: "undefined".to_string(),
            last_state_millis: 0.0_f64,
            bootstrap_nodes,
            endpoint,
        })
    }
}

impl NetWorker for IpcNetWorker {
    /// stop the net worker
    fn stop(self: Box<Self>) -> NetResult<()> {
        self.ipc_relay.stop()?;
        Ok(())
    }

    /// we got a message from holochain core
    /// (just forwards to the internal worker relay)
    fn receive(&mut self, data: Protocol) -> NetResult<()> {
        self.ipc_relay.send(data)?;
        Ok(())
    }

    /// do some upkeep on the internal worker
    /// IPC server state handling / magic
    fn tick(&mut self) -> NetResult<bool> {
        let mut has_done_something = false;

        // Request p2p module's state if its not ready yet
        if &self.last_known_state != "ready" {
            self.priv_request_state()?;
        }

        // Tick the internal worker relay
        if self.ipc_relay.tick()? {
            has_done_something = true;
        }

        // Process back any data sent to us by the ipc_relay to the handler
        if let Ok(data) = self.ipc_relay_receiver.try_recv() {
            has_done_something = true;

            // handle init/config special cases
            if let Ok(msg) = ProtocolWrapper::try_from(&data) {
                match msg {
                    // ipc-server sent us its current state
                    ProtocolWrapper::State(state) => {
                        self.priv_handle_state(state)?;
                    }
                    // ipc-server is requesting us the default config
                    ProtocolWrapper::DefaultConfig(config) => {
                        self.priv_handle_default_config(config)?;
                    }
                    _ => (),
                };
            }

            // Send data back to handler
            (self.handler)(Ok(data))?;

            // When p2p module is ready:
            // - Notify handler that the p2p module is ready
            // - Try connecting to boostrap nodes
            if !self.is_ready && &self.last_known_state == "ready" {
                self.is_ready = true;
                (self.handler)(Ok(Protocol::P2pReady))?;
                self.priv_send_connects()?;
            }
        }

        Ok(has_done_something)
    }

    /// Getter
    fn endpoint(&self) -> Option<String> {
        Some(self.endpoint.clone())
    }
}

// private
impl IpcNetWorker {
    // Send 'Connect to bootstrap nodes' request to Ipc server
    fn priv_send_connects(&mut self) -> NetResult<()> {
        for bs_node in &self.bootstrap_nodes {
            self.ipc_relay.send(
                ProtocolWrapper::Connect(ConnectData {
                    address: bs_node.clone().into(),
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
            self.ipc_relay.send(ProtocolWrapper::RequestState.into())?;
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
            self.ipc_relay
                .send(ProtocolWrapper::RequestDefaultConfig.into())?;
        }
        Ok(())
    }

    /// Handle DefaultConfig Message received from ipc-server.
    /// Pass it back the default config only if it needs configurating
    fn priv_handle_default_config(&mut self, config_msg: ConfigData) -> NetResult<()> {
        if &self.last_known_state == "need_config" {
            self.ipc_relay.send(
                ProtocolWrapper::SetConfig(ConfigData {
                    config: config_msg.config,
                })
                .into(),
            )?;
        }

        Ok(())
    }

    pub const ZMQ_URI_CONFIG: &'static str = r#"{
                "socketType": "zmq",
                "ipcUri": "tcp://127.0.0.1:0",
                "blockConnect": false
            }"#;
}

#[cfg(test)]
mod tests {
    use super::*;

    use holochain_net_connection::protocol::{NamedBinaryData, PongData};

    use holochain_net_ipc::socket::make_test_channels;

    #[test]
    fn it_ipc_networker_zmq_create() {
        IpcNetWorker::new(
            Box::new(|_r| Ok(())),
            &JsonString::from(IpcNetWorker::ZMQ_URI_CONFIG).into(),
        )
        .unwrap();
    }

    #[test]
    fn it_ipc_networker_spawn() {
        if let Err(e) = IpcNetWorker::new(
            Box::new(|_r| Ok(())),
            &json!({
                "socketType": "zmq",
                "spawn": {
                    "cmd": "cargo",
                    "args": [],
                    "workDir": ".",
                    "env": {}
                },
                "blockConnect": false
            })
            .into(),
        ) {
            let e = format!("{:?}", e);
            assert!(e.contains("Invalid argument"), "res: {}", e);
        } else {
            panic!("expected error");
        }
    }

    #[test]
    fn it_ipc_networker_flow() {
        let (handler_send, handler_recv) = mpsc::channel::<Protocol>();
        let (test_struct, test_send, test_recv) = make_test_channels().unwrap();

        let pong = Protocol::Pong(PongData {
            orig: get_millis() - 4.0,
            recv: get_millis() - 2.0,
        });
        let data: NamedBinaryData = (&pong).into();
        test_send
            .send(vec![vec![], vec![], b"pong".to_vec(), data.data])
            .unwrap();

        let mut cli = Box::new(
            IpcNetWorker::new_test(
                Box::new(move |r| {
                    handler_send.send(r?)?;
                    Ok(())
                }),
                test_struct,
            )
            .unwrap(),
        );

        cli.tick().unwrap();

        let res = handler_recv.recv().unwrap();

        assert_eq!(pong, res);

        let json = Protocol::Json(
            json!({
                "method": "state",
                "state": "need_config"
            })
            .into(),
        );
        let data: NamedBinaryData = (&json).into();
        test_send
            .send(vec![vec![], vec![], b"json".to_vec(), data.data])
            .unwrap();

        cli.tick().unwrap();

        let res = handler_recv.recv().unwrap();

        assert_eq!(json, res);

        let res = test_recv.recv().unwrap();
        let res = String::from_utf8_lossy(&res[3]).to_string();
        assert!(res.contains("requestState"));

        let res = test_recv.recv().unwrap();
        let res = String::from_utf8_lossy(&res[3]).to_string();
        assert!(res.contains("requestDefaultConfig"));

        let json = Protocol::Json(
            json!({
                "method": "defaultConfig",
                "config": "test_config"
            })
            .into(),
        );
        let data: NamedBinaryData = (&json).into();
        test_send
            .send(vec![vec![], vec![], b"json".to_vec(), data.data])
            .unwrap();

        cli.tick().unwrap();

        handler_recv.recv().unwrap();

        let res = test_recv.recv().unwrap();
        let res = String::from_utf8_lossy(&res[3]).to_string();
        assert!(res.contains("setConfig"));

        let json = Protocol::Json(
            json!({
                "method": "state",
                "state": "ready",
                "id": "test_id",
                "bindings": ["test_binding_1"]
            })
            .into(),
        );
        let data: NamedBinaryData = (&json).into();
        test_send
            .send(vec![vec![], vec![], b"json".to_vec(), data.data])
            .unwrap();

        cli.tick().unwrap();

        handler_recv.recv().unwrap();

        let res = handler_recv.recv().unwrap();
        assert_eq!(Protocol::P2pReady, res);

        cli.stop().unwrap();
    }
}
