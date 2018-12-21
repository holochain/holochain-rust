//! provides a NetWorker implementation for backend IPC p2p connections

use holochain_core_types::json::JsonString;

use holochain_net_ipc::{
    ipc_client::IpcClient,
    socket::{IpcSocket, MockIpcSocket, TestStruct, ZmqIpcSocket},
    util::get_millis,
};

use holochain_net_connection::{
    net_connection::{
        NetConnection, NetConnectionRelay, NetHandler, NetShutdown, NetWorker, NetWorkerFactory,
    },
    protocol::Protocol,
    protocol_wrapper::{ConfigData, ProtocolWrapper, StateData},
    NetResult,
};

use std::{collections::HashMap, convert::TryFrom, io::Read, sync::mpsc};

use serde_json;

/// a p2p net worker
pub struct IpcNetWorker {
    handler: NetHandler,
    ipc_relay: NetConnectionRelay,
    ipc_relay_receiver: mpsc::Receiver<Protocol>,
    is_ready: bool,
    state: String,
    last_state_millis: f64,
    endpoint: String,
}

impl NetWorker for IpcNetWorker {
    /// stop the net worker
    fn stop(self: Box<Self>) -> NetResult<()> {
        self.ipc_relay.stop()?;
        Ok(())
    }

    fn endpoint(&self) -> Option<String> {
        Some(self.endpoint.clone())
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
            }
        }

        Ok(did_something)
    }
}

impl IpcNetWorker {
    // Constructor with MockIpcSocket on local network
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
            default_local_endpoint.to_string(),
        )
    }

    // Constructor with config as a json string
    pub fn new(handler: NetHandler, config: &JsonString) -> NetResult<Self> {
        // Load config
        let config: serde_json::Value = serde_json::from_str(config.into())?;
        // Only zmq protocol is handled for now
        if config["socketType"] != "zmq" {
            bail!("unexpected socketType: {}", config["socketType"]);
        }
        let block_connect = config["blockConnect"].as_bool().unwrap_or(true);
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
            endpoint,
        )
    }

    // -- private -- //

    /// Constructor with IpcNetWorker instance pointing to a process that we spawn here
    fn priv_new_with_spawn(
        handler: NetHandler,
        cmd: String,
        args: Vec<String>,
        work_dir: String,
        env: HashMap<String, String>,
        block_connect: bool,
    ) -> NetResult<Self> {
        // Create child process (i.e `node n3h`)
        let mut child_process = std::process::Command::new(cmd);
        child_process
            .stdout(std::process::Stdio::piped())
            .args(&args)
            .envs(&env)
            .current_dir(work_dir);
        println!("SPAWN ({:?})", child_process);
        let mut child_process = child_process.spawn()?;

        // Get its stdout
        if child_process.stdout.is_none() {
            bail!("pipe fail");
        }
        let mut stdout = child_process.stdout.unwrap();

        // transport info (zmq uri) for connecting to the ipc socket
        let re_ipc = regex::Regex::new("(?m)^#IPC-BINDING#:(.+)$")?;
        // transport info (multiaddr) for any p2p interface bindings
        let re_p2p = regex::Regex::new("(?m)^#P2P-BINDING#:(.+)$")?;
        // the child process is ready for connections
        let re_ready = regex::Regex::new("#IPC-READY#")?;

        let mut ipc_binding = String::new();
        let mut p2p_bindings: Vec<String> = Vec::new();

        // we need to know when our child process is ready for IPC connections
        // it will run some startup algorithms, and then output some binding
        // info on stdout and finally a `#IPC-READY#` message.
        // collect the binding info, and proceed when `#IPC-READY#` is received
        let mut data: Vec<u8> = Vec::new();
        loop {
            let mut buf: [u8; 4096] = [0; 4096];
            let size = stdout.read(&mut buf)?;
            if size > 0 {
                data.extend_from_slice(&buf[..size]);

                let tmp = String::from_utf8_lossy(&data);
                if re_ready.is_match(&tmp) {
                    for m in re_ipc.captures_iter(&tmp) {
                        ipc_binding = m[1].to_string();
                        break;
                    }
                    for m in re_p2p.captures_iter(&tmp) {
                        p2p_bindings.push(m[1].to_string());
                    }
                    break;
                }
            } else {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }

            if !block_connect {
                break;
            }
        }

        // close the pipe since we can never read from it again...
        child_process.stdout = None;

        println!("READY! {} {:?}", ipc_binding, p2p_bindings);
        let endpoint = ipc_binding.clone();
        // create a new IpcNetWorker that connects to the spawned process
        IpcNetWorker::priv_new(
            handler,
            Box::new(move |h| {
                let mut socket = ZmqIpcSocket::new()?;
                socket.connect(&ipc_binding)?;
                let out: Box<NetWorker> = Box::new(IpcClient::new(h, socket, block_connect)?);
                Ok(out)
            }),
            Some(Box::new(move || {
                child_process.kill().unwrap();
            })),
            endpoint,
        )
    }

    /// Constructor without config, using a NetConnectionRelay as socket
    fn priv_new(
        handler: NetHandler,
        factory: NetWorkerFactory,
        done: NetShutdown,
        endpoint: String,
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
            endpoint,
        })
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
