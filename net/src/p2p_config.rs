use holochain_core_types::{error::HolochainError, json::JsonString};
use std::{collections::HashMap, fs::File, str::FromStr};

//--------------------------------------------------------------------------------------------------
// P2pConfig
//--------------------------------------------------------------------------------------------------

#[derive(Deserialize, Serialize, Clone, Debug, DefaultJson, PartialEq)]
#[serde(tag = "backend_kind", content = "backend_config")]
pub enum P2pConfig {
    #[serde(rename = "IPC")]
    Ipc(P2pIpcConfig),
    #[serde(rename = "MOCK")]
    Mock(P2pMockConfig),
}

#[derive(Deserialize, Serialize, Clone, Debug, DefaultJson, PartialEq)]
pub struct P2pMockConfig {
    pub network_name: String,
}

#[derive(Deserialize, Serialize, Clone, Debug, DefaultJson, PartialEq)]
pub struct P2pIpcConfig {
    #[serde(default)]
    pub spawn: Option<P2pIpcSpawnConfig>,

    #[serde(rename = "socketType")]
    pub socket_type: String,

    #[serde(rename = "ipcUri", default)]
    pub ipc_uri: Option<String>,

    #[serde(rename = "blockConnect", default)]
    pub block_connect: Option<bool>,
}

#[derive(Deserialize, Serialize, Clone, Debug, DefaultJson, PartialEq)]
pub struct P2pIpcSpawnConfig {
    pub cmd: String,
    pub env: HashMap<String, String>,

    #[serde(default)]
    pub args: Vec<String>,

    #[serde(rename = "workDir", default)]
    pub work_dir: String,
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
    pub fn from_file(filepath: &str) -> Self {
        let config_file =
            File::open(filepath).expect("Failed to open filepath on P2pConfig creation.");
        serde_json::from_reader(config_file)
            .expect("file is not a proper JSON of a P2pConfig struct")
    }

    pub fn default_mock(network_name: &'static str) -> Self {
        P2pConfig::from_str(&P2pConfig::default_mock_config(network_name))
            .expect("Invalid backend_config json on P2pConfig creation.")
    }

    pub fn default_ipc() -> Self {
        P2pConfig::from_str(P2pConfig::DEFAULT_IPC_CONFIG)
            .expect("Invalid backend_config json on P2pConfig creation.")
    }
}

// statics
impl P2pConfig {
    pub fn default_mock_config(network_name: &str) -> String {
        format!(
            r#"{{
            "backend_kind": "MOCK",
            "backend_config": {{
                "network_name": "{}"
            }}
        }}"#,
            network_name
        )
    }

    pub const DEFAULT_IPC_CONFIG: &'static str = r#"
    {
      "backend_kind": "IPC",
      "backend_config": {
        "socketType": "zmq",
        "spawn": {
          "cmd": "node",
          "env": {
            "N3H_HACK_MODE": "1",
            "N3H_IPC_SOCKET": "tcp://127.0.0.1:*"
          }
        }
      }
    }"#;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_json_round_trip() {
        let p2p_config =
            P2pConfig::from_str(&P2pConfig::default_mock_config("it_can_json_round_trip")).unwrap();
        let json_str = p2p_config.as_str();
        let p2p_config_2 = P2pConfig::from_str(&json_str).unwrap();
        assert_eq!(p2p_config, p2p_config_2);
        assert_eq!(
            p2p_config,
            P2pConfig::default_mock("it_can_json_round_trip")
        );
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
