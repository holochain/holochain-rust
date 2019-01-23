use holochain_net::{p2p_config::*, p2p_network::P2pNetwork};
use holochain_net_connection::{
    json_protocol::JsonProtocol, net_connection::NetSend, protocol::Protocol, NetResult,
};
use std::{convert::TryFrom, sync::mpsc};

static TIMEOUT_MS: usize = 5000;

pub struct P2pNode {
    // Need to hold the tempdir to keep it alive, otherwise we will get a dir error.
    _maybe_temp_dir: Option<tempfile::TempDir>,
    p2p_connection: P2pNetwork,
    receiver: mpsc::Receiver<Protocol>,
    pub config: P2pConfig,
    pub name: String,
}

impl P2pNode {
    /// Private constructor
    #[cfg_attr(tarpaulin, skip)]
    pub fn new_with_config(name_arg: String, config: &P2pConfig, _maybe_temp_dir: Option<tempfile::TempDir>) -> Self {
        // use a mpsc channel for messaging between p2p connection and main thread
        let (sender, receiver) = mpsc::channel::<Protocol>();
        // create a new P2pNetwork instance with the handler that will send the received Protocol to a channel

        let name = name_arg.clone();

        let p2p_connection = P2pNetwork::new(
            Box::new(move |r| {
                println!("<<< P2pNode({}) handler: {:?}", name_arg, r);
                sender.send(r?)?;
                Ok(())
            }),
            &config,
        )
        .expect("Failed to create P2pNetwork");

        P2pNode {
            _maybe_temp_dir,
            p2p_connection,
            receiver,
            config: config.clone(),
            name,
        }
    }

    // Constructor for an in-memory P2P Network
    #[cfg_attr(tarpaulin, skip)]
    pub fn new_with_unique_memory_network(name: String) -> Self {
        let config = P2pConfig::new_with_unique_memory_backend();
        return P2pNode::new_with_config(name, &config, None);
    }

    // Constructor for an IPC node that uses an existing n3h process and a temp folder
    #[cfg_attr(tarpaulin, skip)]
    pub fn new_with_uri_ipc_network(name: String, ipc_binding: &str) -> Self {
        let p2p_config = P2pConfig::default_ipc_uri(Some(ipc_binding));
        return P2pNode::new_with_config(name, &p2p_config, None);
    }

    // Constructor for an IPC node that spawns and uses a n3h process and a temp folder
    #[cfg_attr(tarpaulin, skip)]
    pub fn new_with_spawn_ipc_network(
        name: String,
        n3h_path: &str,
        maybe_config_filepath: Option<&str>,
        bootstrap_nodes: Vec<String>,
    ) -> Self {
        let (p2p_config, temp_dir) =
            create_ipc_config(n3h_path, maybe_config_filepath, bootstrap_nodes);
        return P2pNode::new_with_config(name, &p2p_config, Some(temp_dir));
    }

    // See if there is a message to receive
    #[cfg_attr(tarpaulin, skip)]
    pub fn try_recv(&mut self) -> NetResult<JsonProtocol> {
        let data = self.receiver.try_recv()?;
        // Print non-ping messages
        match data {
            Protocol::NamedBinary(_) => println!("<< P2pNode({}) recv: {:?}", self.name, data),
            Protocol::Json(_) => println!("<< P2pNode({}) recv: {:?}", self.name, data),
            _ => (),
        };

        match JsonProtocol::try_from(&data) {
            Ok(r) => Ok(r),
            Err(e) => {
                let s = format!("{:?}", e);
                if !s.contains("Empty") && !s.contains("Pong(PongData") {
                    println!("###### Received parse error ###### {} {:?}", s, data);
                }
                Err(e)
            }
        }
    }

    /// Wait for receiving a message corresponding to predicate
    #[cfg_attr(tarpaulin, skip)]
    pub fn wait(
        &mut self,
        predicate: Box<dyn Fn(&JsonProtocol) -> bool>,
    ) -> NetResult<JsonProtocol> {
        let mut time_ms: usize = 0;
        loop {
            let mut did_something = false;

            if let Ok(p2p_msg) = self.try_recv() {
                println!("P2pNode({})::wait() - received: {:?}", self.name, p2p_msg);
                did_something = true;
                if predicate(&p2p_msg) {
                    println!("\t P2pNode({})::wait() - match", self.name);
                    return Ok(p2p_msg);
                } else {
                    println!("\t P2pNode({})::wait() - NO match", self.name);
                }
            }

            if !did_something {
                std::thread::sleep(std::time::Duration::from_millis(10));
                time_ms += 10;
                if time_ms > TIMEOUT_MS {
                    panic!("P2pNode({})::wait() has TIMEOUT", self.name);
                }
            }
        }
    }

    // Stop node
    #[cfg_attr(tarpaulin, skip)]
    pub fn stop(self) {
        self.p2p_connection
            .stop()
            .expect("Failed to stop p2p connection properly");
    }

    /// Getter of the endpoint of its connection
    #[cfg_attr(tarpaulin, skip)]
    pub fn endpoint(&self) -> String {
        self.p2p_connection.endpoint()
    }
}

impl NetSend for P2pNode {
    /// send a Protocol message to the p2p network instance
    fn send(&mut self, data: Protocol) -> NetResult<()> {
        // Debugging code (do not delete)
        println!(">> P2pNode({}) send: {:?}", self.name, data);
        self.p2p_connection.send(data)
    }
}

//--------------------------------------------------------------------------------------------------
// create_ipc_config
//--------------------------------------------------------------------------------------------------

/// Create an P2pConfig for an IPC node that uses n3h and a temp folder
#[cfg_attr(tarpaulin, skip)]
fn create_ipc_config(
    n3h_path: &str,
    maybe_config_filepath: Option<&str>,
    bootstrap_nodes: Vec<String>,
) -> (P2pConfig, tempfile::TempDir) {
    // Create temp directory
    let dir_ref = tempfile::tempdir().expect("Failed to created a temp directory.");
    let dir = dir_ref.path().to_string_lossy().to_string();

    println!("create_ipc_config() dir = {}\n", dir);

    // Create config
    let config = match maybe_config_filepath {
        Some(filepath) => {
            // Get config from file
            let p2p_config = P2pConfig::from_file(filepath);
            assert_eq!(p2p_config.backend_kind, P2pBackendKind::IPC);
            // complement missing fields
            serde_json::from_value(json!({
            "backend_kind": String::from(p2p_config.backend_kind),
            "backend_config":
            {
                "socketType": p2p_config.backend_config["socketType"],
                "bootstrapNodes": bootstrap_nodes,
                "spawn":
                {
                    "cmd": p2p_config.backend_config["spawn"]["cmd"],
                    "args": [
                        format!("{}/packages/n3h/bin/n3h", n3h_path)
                    ],
                    "workDir": dir.clone(),
                    "env": {
                        "N3H_MODE": p2p_config.backend_config["spawn"]["env"]["N3H_MODE"],
                        "N3H_WORK_DIR": dir.clone(),
                        "N3H_IPC_SOCKET": p2p_config.backend_config["spawn"]["env"]["N3H_IPC_SOCKET"],
                    }
                },
            }})).unwrap()
        }
        None => {
            // use default config
            serde_json::from_value(json!({
            "backend_kind": "IPC",
            "backend_config":
            {
                "socketType": "zmq",
                "bootstrapNodes": bootstrap_nodes,
                "spawn":
                {
                    "cmd": "node",
                    "args": [
                        format!("{}/packages/n3h/bin/n3h", n3h_path)
                    ],
                    "workDir": dir.clone(),
                    "env": {
                        "N3H_MODE": "HACK",
                        "N3H_WORK_DIR": dir.clone(),
                        "N3H_IPC_SOCKET": "tcp://127.0.0.1:*",
                }
            },
            }}))
            .unwrap()
        }
    };
    return (config, dir_ref);
}
