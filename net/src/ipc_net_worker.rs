//! provides a NetWorker implementation for backend IPC p2p connections

use holochain_core_types::json::JsonString;

use holochain_net_ipc::{
    ipc_client::IpcClient,
    socket::{IpcSocket, MockIpcSocket, TestStruct, ZmqIpcSocket},
    spawn,
    util::get_millis,
};

use holochain_net_connection::{
    net_connection::{
        NetConnection, NetConnectionRelay, NetHandler, NetShutdown, NetWorker, NetWorkerFactory,
    },
    protocol::Protocol,
    protocol_wrapper::{ConfigData, ConnectData, ProtocolWrapper, StateData},
    NetResult,
};

use std::{collections::HashMap, convert::TryFrom, sync::mpsc};

use serde_json;

/// a p2p net worker
pub struct IpcNetWorker {
    handler: NetHandler,
    ipc_relay: NetConnectionRelay,
    ipc_relay_receiver: mpsc::Receiver<Protocol>,
    is_ready: bool,
    state: String,
    last_state_millis: f64,
    bootstrap_nodes: Vec<String>,
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
    fn tick(&mut self) -> NetResult<bool> {
        let mut did_something = false;

        if &self.state != "ready" {
            self.priv_check_init()?;
        }

        if self.ipc_relay.tick()? {
            did_something = true;
        }

        if let Ok(data) = self.ipc_relay_receiver.try_recv() {
            did_something = true;

            if let Ok(wrap) = ProtocolWrapper::try_from(&data) {
                match wrap {
                    ProtocolWrapper::State(s) => {
                        self.priv_handle_state(s)?;
                    }
                    ProtocolWrapper::DefaultConfig(c) => {
                        self.priv_handle_default_config(c)?;
                    }
                    _ => (),
                };
            }

            (self.handler)(Ok(data))?;

            if !self.is_ready && &self.state == "ready" {
                self.is_ready = true;
                (self.handler)(Ok(Protocol::P2pReady))?;
                self.priv_send_connects()?;
            }
        }

        Ok(did_something)
    }
}

impl IpcNetWorker {
    pub fn new_test(handler: NetHandler, test_struct: TestStruct) -> NetResult<Self> {
        IpcNetWorker::priv_new(
            handler,
            Box::new(move |h| {
                let mut socket = MockIpcSocket::new_test(test_struct)?;
                socket.connect("tcp://127.0.0.1:0")?;
                let out: Box<NetWorker> = Box::new(IpcClient::new(h, socket, true)?);
                Ok(out)
            }),
            None,
            vec![],
        )
    }

    pub fn new(handler: NetHandler, config: &JsonString) -> NetResult<Self> {
        let config: serde_json::Value = serde_json::from_str(config.into())?;
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
            if let Some(s) = config["spawn"].as_object() {
                if s["cmd"].is_string()
                    && s["args"].is_array()
                    && s["workDir"].is_string()
                    && s["env"].is_object()
                {
                    let env: HashMap<String, String> = s["env"]
                        .as_object()
                        .unwrap()
                        .iter()
                        .map(|(k, v)| (k.to_string(), v.as_str().unwrap().to_string()))
                        .collect();
                    return IpcNetWorker::priv_spawn(
                        handler,
                        s["cmd"].as_str().unwrap().to_string(),
                        s["args"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|i| i.as_str().unwrap_or_default().to_string())
                            .collect(),
                        s["workDir"].as_str().unwrap().to_string(),
                        env,
                        block_connect,
                        bootstrap_nodes,
                    );
                } else {
                    bail!("config.spawn requires 'cmd', 'args', 'workDir', and 'env'");
                }
            }
            bail!("config.ipcUri is required");
        }
        let uri = config["ipcUri"].as_str().unwrap().to_string();
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
        )
    }

    // -- private -- //

    /// create a new IpcNetWorker instance pointing to a process that we spawn here
    fn priv_spawn(
        handler: NetHandler,
        cmd: String,
        args: Vec<String>,
        work_dir: String,
        env: HashMap<String, String>,
        block_connect: bool,
        bootstrap_nodes: Vec<String>,
    ) -> NetResult<Self> {
        let spawn_result = spawn::ipc_spawn(cmd, args, work_dir, env, block_connect)?;

        let ipc_binding = spawn_result.ipc_binding;
        let kill = spawn_result.kill;

        IpcNetWorker::priv_new(
            handler,
            Box::new(move |h| {
                let mut socket = ZmqIpcSocket::new()?;
                socket.connect(&ipc_binding)?;
                let out: Box<NetWorker> = Box::new(IpcClient::new(h, socket, block_connect)?);
                Ok(out)
            }),
            kill,
            bootstrap_nodes,
        )
    }

    /// create a new IpcNetWorker instance
    fn priv_new(
        handler: NetHandler,
        factory: NetWorkerFactory,
        done: NetShutdown,
        bootstrap_nodes: Vec<String>,
    ) -> NetResult<Self> {
        let (ipc_sender, ipc_relay_receiver) = mpsc::channel::<Protocol>();

        let ipc_relay = NetConnectionRelay::new(
            Box::new(move |r| {
                ipc_sender.send(r?)?;
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

            state: "undefined".to_string(),

            last_state_millis: 0.0_f64,

            bootstrap_nodes,
        })
    }

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

    /// send a ping twice per second
    fn priv_check_init(&mut self) -> NetResult<()> {
        let now = get_millis();

        if now - self.last_state_millis > 500.0 {
            self.ipc_relay.send(ProtocolWrapper::RequestState.into())?;
            self.last_state_millis = now;
        }

        Ok(())
    }

    /// if the internal worker needs configuration, fetch the default config
    fn priv_handle_state(&mut self, state: StateData) -> NetResult<()> {
        self.state = state.state;

        if &self.state == "need_config" {
            self.ipc_relay
                .send(ProtocolWrapper::RequestDefaultConfig.into())?;
        }

        Ok(())
    }

    /// if the internal worker still needs configuration,
    /// pass it back the default config
    fn priv_handle_default_config(&mut self, config: ConfigData) -> NetResult<()> {
        if &self.state == "need_config" {
            self.ipc_relay.send(
                ProtocolWrapper::SetConfig(ConfigData {
                    config: config.config,
                })
                .into(),
            )?;
        }

        Ok(())
    }
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
            &json!({
                "socketType": "zmq",
                "ipcUri": "tcp://127.0.0.1:0",
                "blockConnect": false
            })
            .into(),
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
