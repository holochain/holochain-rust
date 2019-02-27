//! provides a NetWorker implementation for backend IPC p2p connections

use holochain_core_types::json::JsonString;

use crate::ipc::{spawn, util::get_millis, Transport, TransportEvent, TransportWss};

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
    handler: NetHandler,
    socket: TransportWss<std::net::TcpStream>,
    ipc_uri: String,

    done: NetShutdown,

    is_ready: bool,

    last_known_state: String,
    last_state_millis: f64,

    bootstrap_nodes: Vec<String>,
    endpoint: String,

    log: TweetProxy,
}

// Constructors
impl IpcNetWorker {
    // Constructor with config as a json string
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
                enduser_config,
                env,
                bootstrap_nodes,
            );
        }
        // create a new IpcNetWorker that connects to the given 'ipcUri'
        let uri = config["ipcUri"].as_str().unwrap().to_string();
        let endpoint = uri.clone();
        IpcNetWorker::priv_new(handler, uri, None, bootstrap_nodes, endpoint)
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
        config: String,
        env: HashMap<String, String>,
        bootstrap_nodes: Vec<String>,
    ) -> NetResult<Self> {
        // Spawn a process with given `cmd` that we will have an IPC connection with
        let spawn_result = spawn::ipc_spawn(cmd, args, work_dir, config, env, true)?;
        // Get spawn result info
        let ipc_binding = spawn_result.ipc_binding;
        let kill = spawn_result.kill;
        let endpoint = ipc_binding.clone();

        // Done
        IpcNetWorker::priv_new(handler, ipc_binding, kill, bootstrap_nodes, endpoint)
    }

    /// Constructor without config
    fn priv_new(
        handler: NetHandler,
        ipc_uri: String,
        done: NetShutdown,
        bootstrap_nodes: Vec<String>,
        endpoint: String,
    ) -> NetResult<Self> {
        let log = TweetProxy::new("IpcNetWorker");
        log.i(&format!("connect to uri {}", ipc_uri));

        let mut socket = TransportWss::with_std_tcp_stream();
        wait_connect(&mut socket, &ipc_uri)?;

        log.i("connection success");

        Ok(IpcNetWorker {
            handler,
            socket,
            ipc_uri,
            done,
            is_ready: false,
            last_known_state: "undefined".to_string(),
            last_state_millis: 0.0_f64,
            bootstrap_nodes,
            endpoint,
            log,
        })
    }
}

fn wait_connect(
    socket: &mut TransportWss<std::net::TcpStream>,
    uri: &str,
) -> NetResult<Vec<TransportEvent>> {
    let mut out = Vec::new();

    socket.connect(&uri)?;

    let start = std::time::Instant::now();
    while start.elapsed().as_millis() < 5000 {
        let (_did_work, evt_lst) = socket.poll()?;
        for evt in evt_lst {
            match evt {
                TransportEvent::Connect(_id) => {
                    return Ok(out);
                }
                _ => out.push(evt),
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(3));
    }

    bail!("ipc connection timeout");
}

impl NetWorker for IpcNetWorker {
    /// stop the net worker
    fn stop(mut self: Box<Self>) -> NetResult<()> {
        self.socket.close_all()?;
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
        self.socket.send_all(&data)?;

        Ok(())
    }

    /// do some upkeep on the internal worker
    /// IPC server state handling / magic
    fn tick(&mut self) -> NetResult<bool> {
        // Request p2p module's state if its not ready yet
        if &self.last_known_state != "ready" {
            self.priv_request_state()?;
        }

        let (did_work, evt_lst) = self.socket.poll()?;
        for evt in evt_lst {
            match evt {
                TransportEvent::TransportError(_id, e) => {
                    self.log.e(&format!("ipc ws error {:?}", e));
                    self.socket.close_all()?;
                    wait_connect(&mut self.socket, &self.ipc_uri)?;
                }
                TransportEvent::Connect(_id) => {
                    // don't need to do anything here
                }
                TransportEvent::Close(_id) => {
                    self.log.e("ipc ws closed");
                    self.socket.close_all()?;
                    wait_connect(&mut self.socket, &self.ipc_uri)?;
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
                    if !self.is_ready && &self.last_known_state == "ready" {
                        self.is_ready = true;
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
        Some(self.endpoint.clone())
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

    pub const ZMQ_URI_CONFIG: &'static str = r#"{
                "socketType": "zmq",
                "ipcUri": "tcp://127.0.0.1:0",
                "blockConnect": false
            }"#;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        connection::protocol::{NamedBinaryData, PongData},
        ipc::socket::make_test_channels,
        p2p_config::P2pConfig,
    };

    #[test]
    fn it_ipc_networker_zmq_create() {
        IpcNetWorker::new(
            Box::new(|_r| Ok(())),
            &JsonString::from(IpcNetWorker::ZMQ_URI_CONFIG).into(),
            P2pConfig::default_end_user_config().to_string(),
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
            P2pConfig::default_end_user_config().to_string(),
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
*/
