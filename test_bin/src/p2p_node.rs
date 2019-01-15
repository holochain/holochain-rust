use holochain_net::{p2p_config::*, p2p_network::P2pNetwork};
use holochain_net_connection::{
    net_connection::NetSend, protocol::Protocol, protocol_wrapper::ProtocolWrapper, NetResult,
};
use std::{convert::TryFrom, sync::mpsc};

static TIMEOUT_MS: usize = 5000;

pub struct P2pNode {
    // Need to hold the tempdir to keep it alive, otherwise we will get a dir error.
    _maybe_temp_dir: Option<tempfile::TempDir>,
    p2p_connection: P2pNetwork,
    receiver: mpsc::Receiver<Protocol>,
    pub config: P2pConfig,
}

impl P2pNode {
    /// Private constructor
    #[cfg_attr(tarpaulin, skip)]
    pub fn new_with_config(config: &P2pConfig, _maybe_temp_dir: Option<tempfile::TempDir>) -> Self {
        // use a mpsc channel for messaging between p2p connection and main thread
        let (sender, receiver) = mpsc::channel::<Protocol>();
        // create a new P2pNetwork instance with the handler that will send the received Protocol to a channel
        let p2p_connection = P2pNetwork::new(
            Box::new(move |r| {
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
        }
    }

    // Constructor for a mock P2P Network
    #[cfg_attr(tarpaulin, skip)]
    pub fn new_mock() -> Self {
        let config = P2pConfig::unique_mock();
        return P2pNode::new_with_config(&config, None);
    }

    // Constructor for an IPC node that uses an existing n3h process and a temp folder
    #[cfg_attr(tarpaulin, skip)]
    pub fn new_ipc_with_uri(ipc_binding: &str) -> Self {
        let p2p_config = P2pConfig::default_ipc_uri(Some(ipc_binding));
        return P2pNode::new_with_config(&p2p_config, None);
    }

    // Constructor for an IPC node that spawns and uses a n3h process and a temp folder
    #[cfg_attr(tarpaulin, skip)]
    pub fn new_ipc_spawn(
        n3h_path: &str,
        maybe_config_filepath: Option<&str>,
        bootstrap_nodes: Vec<String>,
    ) -> Self {
        let (p2p_config, temp_dir) =
            create_ipc_config(n3h_path, maybe_config_filepath, bootstrap_nodes);
        return P2pNode::new_with_config(&p2p_config, Some(temp_dir));
    }

    // See if there is a message to receive
    #[cfg_attr(tarpaulin, skip)]
    pub fn try_recv(&mut self) -> NetResult<ProtocolWrapper> {
        let data = self.receiver.try_recv()?;
        // Print non-ping messages
        match data {
            Protocol::NamedBinary(_) => println!("<< P2pNode recv: {:?}", data),
            Protocol::Json(_) => println!("<< P2pNode recv: {:?}", data),
            _ => (),
        };

        match ProtocolWrapper::try_from(&data) {
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
        predicate: Box<dyn Fn(&ProtocolWrapper) -> bool>,
    ) -> NetResult<ProtocolWrapper> {
        let mut time_ms: usize = 0;
        loop {
            let mut did_something = false;

            if let Ok(p2p_msg) = self.try_recv() {
                println!("P2pNode::wait() received: {:?}", p2p_msg);
                did_something = true;
                if predicate(&p2p_msg) {
                    println!("P2pNode::wait() found match");
                    return Ok(p2p_msg);
                } else {
                    println!("P2pNode::wait() found NOT match");
                }
            }

            if !did_something {
                std::thread::sleep(std::time::Duration::from_millis(10));
                time_ms += 10;
                if time_ms > TIMEOUT_MS {
                    panic!("P2pNode::wait() has TIMEOUT");
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
        // println!(">> P2pNode send: {:?}", data);
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
