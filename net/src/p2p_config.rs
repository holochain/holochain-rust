use holochain_core_types::{error::HolochainError, json::JsonString};
use snowflake;
use std::{fs::File, str::FromStr, io::prelude::*};

//--------------------------------------------------------------------------------------------------
// P2pBackendKind
//--------------------------------------------------------------------------------------------------

#[derive(Deserialize, Serialize, Clone, Debug, DefaultJson, PartialEq, Eq)]
pub enum P2pBackendKind {
    MEMORY,
    IPC,
}

impl FromStr for P2pBackendKind {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "MEMORY" => Ok(P2pBackendKind::MEMORY),
            "IPC" => Ok(P2pBackendKind::IPC),
            _ => Err(()),
        }
    }
}

impl From<P2pBackendKind> for String {
    fn from(kind: P2pBackendKind) -> String {
        String::from(match kind {
            P2pBackendKind::MEMORY => "MEMORY",
            P2pBackendKind::IPC => "IPC",
        })
    }
}

impl From<String> for P2pBackendKind {
    fn from(s: String) -> P2pBackendKind {
        P2pBackendKind::from_str(&s).expect("could not convert String to P2pBackendKind")
    }
}

impl From<&'static str> for P2pBackendKind {
    fn from(s: &str) -> P2pBackendKind {
        P2pBackendKind::from(String::from(s))
    }
}

//--------------------------------------------------------------------------------------------------
// P2pConfig
//--------------------------------------------------------------------------------------------------

#[derive(Deserialize, Serialize, Clone, Debug, DefaultJson, PartialEq)]
pub struct P2pConfig {
    pub backend_kind: P2pBackendKind,
    pub backend_config: serde_json::Value,
    pub maybe_end_user_config: Option<serde_json::Value>
}

// Conversions
impl FromStr for P2pConfig {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s).map_err(|err| err.to_string())
    }
}
impl P2pConfig {
    pub fn as_str(&self) -> String {
        // unwrap() is safe since there is no way this can fail
        // since this struct derives from Serialize.
        serde_json::to_string(self).unwrap()
    }
}

// Constructors
impl P2pConfig {
    pub fn new(
        backend_kind: P2pBackendKind,
        backend_config: &str,
        maybe_end_user_config: Option<serde_json::Value>,
    ) -> Self {
        P2pConfig {
            backend_kind,
            backend_config: serde_json::from_str(backend_config)
                .expect("Invalid backend_config json on P2pConfig creation."),
            maybe_end_user_config,
        }
    }

    pub fn from_file(filepath: &str) -> Self {
        let config_file =
            File::open(filepath).expect("Failed to open filepath on P2pConfig creation.");
        serde_json::from_reader(config_file)
            .expect("file is not a proper JSON of a P2pConfig struct")
    }

    pub fn default_ipc_spawn() -> Self {
        P2pConfig::from_str(P2pConfig::DEFAULT_IPC_SPAWN_CONFIG)
            .expect("Invalid backend_config json on P2pConfig creation.")
    }

    pub fn new_ipc_uri(
        maybe_ipc_binding: Option<String>,
        bootstrap_nodes: &Vec<String>,
        maybe_end_user_config_filepath: Option<String>,

    ) -> Self {
        let backend_config = json!({
            "socketType": "zmq",
            "blockConnect": false,
            "bootstrapNodes": bootstrap_nodes,
            "ipcUri": maybe_ipc_binding
        })
            .to_string();
        P2pConfig::new(
            P2pBackendKind::IPC,
            &backend_config,
            Some(P2pConfig::load_end_user_config(maybe_end_user_config_filepath)),
        )
    }

    pub fn default_ipc_uri(maybe_ipc_binding: Option<&str>) -> Self {
        match maybe_ipc_binding {
            None => P2pConfig::from_str(P2pConfig::DEFAULT_IPC_URI_CONFIG)
                .expect("Invalid backend_config json on P2pConfig creation."),
            Some(ipc_binding) => {
                let backend_config = json!({
                    "socketType": "zmq",
                    "blockConnect": false,
                    "ipcUri": ipc_binding
                })
                .to_string();
                P2pConfig::new(
                    P2pBackendKind::IPC,
                    &backend_config,
                    Some(P2pConfig::default_n3h_end_user_config()),
                )
            }
        }
    }

    pub fn new_with_memory_backend(server_name: &str) -> Self {
        P2pConfig::new(
            P2pBackendKind::MEMORY,
            &Self::memory_backend_string(server_name),
            None,
        )
    }

    pub fn new_with_unique_memory_backend() -> Self {
        Self::new_with_memory_backend(&format!(
            "memory-auto-{}",
            snowflake::ProcessUniqueId::new().to_string()
        ))
    }

    pub fn unique_memory_backend_string() -> String {
        Self::memory_backend_string(&format!(
            "memory-auto-{}",
            snowflake::ProcessUniqueId::new().to_string()
        ))
    }

    pub fn memory_backend_string(server_name: &str) -> String {
        format!(
            r#"{{
            "serverName": "{}"
            }}"#,
            server_name
        )
    }
}

impl P2pConfig {
    pub fn default_n3h_end_user_config() -> serde_json::Value {
        json!({
          "webproxy": {
            "connection": {
              "rsaBits": 1024,
              "bind": [
                "wss://0.0.0.0:0/"
              ]
            },
            "wssAdvertise": "auto",
            "wssRelayPeers": null
          }
        })
    }

    pub fn load_end_user_config(maybe_end_user_config_filepath: Option<String>) -> serde_json::Value {
        match maybe_end_user_config_filepath {
            None => P2pConfig::default_n3h_end_user_config(),
            Some(filepath) => {
                let mut file = File::open(filepath);
                if let Err(_) = file {
                    return P2pConfig::default_n3h_end_user_config();
                }
                let mut contents = String::new();
                let res = file.unwrap().read_to_string(&mut contents);
                if let Err(_) = res {
                    return P2pConfig::default_n3h_end_user_config();
                }
                json!(&contents)
            },
        }
    }
}

// statics
impl P2pConfig {
    pub const DEFAULT_IPC_SPAWN_CONFIG: &'static str = r#"
    {
      "backend_kind": "IPC",
      "backend_config": {
        "socketType": "zmq",
        "spawn": {
          "cmd": "node",
          "env": {
            "N3H_MODE": "HACK",
            "N3H_IPC_SOCKET": "tcp://127.0.0.1:*"
          }
        }
      }
    }"#;

    pub const DEFAULT_IPC_URI_CONFIG: &'static str = r#"
    {
      "backend_kind": "IPC",
      "backend_config": {
        "socketType": "zmq",
        "ipcUri": "tcp://127.0.0.1:0",
        "blockConnect": false
      }
    }"#;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_json_round_trip() {
        let server_name = "memory_test";
        let p2p_config =
            P2pConfig::from_str(&P2pConfig::new_with_memory_backend(server_name).as_str()).unwrap();
        let json_str = p2p_config.as_str();
        let p2p_config_2 = P2pConfig::from_str(&json_str).unwrap();
        assert_eq!(p2p_config, p2p_config_2);
        assert_eq!(p2p_config, P2pConfig::new_with_memory_backend(server_name));
    }

    #[test]
    fn it_should_fail_bad_backend_kind() {
        let res = P2pConfig::from_str(
            r#"{
            "backend_kind": "BAD",
            "backend_config": "",
            }"#,
        );
        assert!(res.is_err());
        let err = format!("{:?}", res.err().unwrap());
        assert!(err.contains("unknown variant `BAD`"), "e = {}", err);
    }
}
