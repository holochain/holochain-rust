use crate::{sim2h_worker::Sim2hConfig, NEW_RELIC_LICENSE_KEY};
use holochain_json_api::{error::JsonError, json::JsonString};
use lib3h::engine::{EngineConfig, GatewayId, TransportConfig};
use lib3h_protocol::uri::Lib3hUri;
use snowflake;
use std::{fs::File, io::prelude::*, str::FromStr};
use url::Url;
//--------------------------------------------------------------------------------------------------
// P2pBackendKind
//--------------------------------------------------------------------------------------------------

#[derive(Deserialize, Serialize, Clone, Debug, DefaultJson, PartialEq, Eq)]
pub enum P2pBackendKind {
    GhostEngineMemory,
    LIB3H,
    SIM2H,
    LegacyInMemory,
}

impl FromStr for P2pBackendKind {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GhostEngineMemory" => Ok(P2pBackendKind::GhostEngineMemory),
            "LIB3H" => Ok(P2pBackendKind::LIB3H),
            "SIM2H" => Ok(P2pBackendKind::SIM2H),
            "LegacyInMemory" => Ok(P2pBackendKind::LegacyInMemory),
            _ => Err(()),
        }
    }
}

impl From<P2pBackendKind> for String {
    fn from(kind: P2pBackendKind) -> String {
        String::from(match kind {
            P2pBackendKind::GhostEngineMemory => "GhostEngineMemory",
            P2pBackendKind::LIB3H => "LIB3H",
            P2pBackendKind::SIM2H => "SIM2H",
            P2pBackendKind::LegacyInMemory => "LegacyInMemory",
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
pub enum BackendConfig {
    Json(serde_json::Value),
    Lib3h(EngineConfig),
    Memory(EngineConfig),
    Sim2h(Sim2hConfig),
}

#[derive(Deserialize, Serialize, Clone, Debug, DefaultJson, PartialEq)]
pub struct P2pConfig {
    pub backend_kind: P2pBackendKind,
    pub backend_config: BackendConfig,
    pub maybe_end_user_config: Option<serde_json::Value>,
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
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_NET)]
impl P2pConfig {
    pub fn new(
        backend_kind: P2pBackendKind,
        backend_config: BackendConfig,
        maybe_end_user_config: Option<serde_json::Value>,
    ) -> Self {
        P2pConfig {
            backend_kind,
            backend_config,
            maybe_end_user_config,
        }
    }

    pub fn from_file(filepath: &str) -> Self {
        let config_file =
            File::open(filepath).expect("Failed to open filepath on P2pConfig creation.");
        serde_json::from_reader(config_file)
            .expect("file is not a proper JSON of a P2pConfig struct")
    }

    pub fn default_lib3h() -> Self {
        P2pConfig::from_str(P2pConfig::DEFAULT_LIB3H_CONFIG)
            .expect("Invalid backend_config json on P2pConfig creation.")
    }

    pub fn new_with_memory_backend(server_name: &str) -> Self {
        P2pConfig::new(
            P2pBackendKind::LegacyInMemory,
            BackendConfig::Json(Self::memory_backend_json(server_name)),
            None,
        )
    }

    pub fn new_with_sim2h_backend(sim2h_url: &str) -> Self {
        P2pConfig::new(
            P2pBackendKind::SIM2H,
            BackendConfig::Sim2h(Sim2hConfig {
                sim2h_url: sim2h_url.into(),
            }),
            None,
        )
    }

    pub fn new_with_memory_lib3h_backend(server_name: &str, bootstrap_nodes: Vec<Url>) -> Self {
        let _host_name = server_name
            .replace(":", "_")
            .replace(" ", "_")
            .replace(",", "_");

        P2pConfig::new(
            P2pBackendKind::GhostEngineMemory,
            BackendConfig::Memory(EngineConfig {
                network_id: GatewayId {
                    nickname: server_name.into(),
                    id: server_name.into(),
                },
                //need to fix the transport configs
                transport_configs: vec![TransportConfig::Memory(server_name.to_string())],
                bootstrap_nodes: bootstrap_nodes
                    .iter()
                    .map(|url| url.clone().into())
                    .collect(),
                work_dir: "".into(),
                log_level: 'd',
                bind_url: Lib3hUri::with_undefined(),
                dht_custom_config: vec![],
                dht_timeout_threshold: 2000,
                dht_gossip_interval: 20,
            }),
            None,
        )
    }

    pub fn new_with_unique_memory_backend() -> Self {
        Self::new_with_memory_backend(&format!(
            "memory-auto-{}",
            snowflake::ProcessUniqueId::new().to_string()
        ))
    }

    pub fn new_with_unique_memory_backend_bootstrap_nodes(bootstrap_nodes: Vec<url::Url>) -> Self {
        Self::new_with_memory_lib3h_backend(
            &format!(
                "memory-auto-{}",
                snowflake::ProcessUniqueId::new().to_string()
            ),
            bootstrap_nodes,
        )
    }

    pub fn unique_memory_backend_json() -> serde_json::Value {
        Self::memory_backend_json(&format!(
            "memory-auto-{}",
            snowflake::ProcessUniqueId::new().to_string()
        ))
    }

    pub fn memory_backend_json(server_name: &str) -> serde_json::Value {
        json!({ "serverName": server_name })
    }
}

/// end_user config
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_NET)]
impl P2pConfig {
    pub fn default_end_user_config() -> serde_json::Value {
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

    pub fn load_end_user_config(
        maybe_end_user_config_filepath: Option<String>,
    ) -> serde_json::Value {
        fn load_config_file(filepath: String) -> Result<serde_json::Value, std::io::Error> {
            let mut file = File::open(filepath)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            let res = serde_json::from_str(&contents);
            res.map_err(|_e| std::io::Error::new(std::io::ErrorKind::Other, "serde fail"))
        }

        match maybe_end_user_config_filepath {
            None => P2pConfig::default_end_user_config(),
            Some(filepath) => match load_config_file(filepath) {
                Err(_) => return P2pConfig::default_end_user_config(),
                Ok(json) => json,
            },
        }
    }
}

/// Utility functions to extract config elements
impl P2pConfig {
    pub fn real_engine_config(self) -> Option<EngineConfig> {
        match self.backend_config {
            BackendConfig::Lib3h(config) => Some(config),
            BackendConfig::Memory(config) => Some(config),
            BackendConfig::Json(_) => None,
            BackendConfig::Sim2h(_) => None,
        }
    }
}

/// statics
impl P2pConfig {
    pub const DEFAULT_LIB3H_CONFIG: &'static str = r#"
    {
      "backend_kind": "LIB3H",
      "backend_config": {
        "socketType": "ws",
        "logLevel": "i"
      }
    }"#;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_json_round_trip() {
        let server_name = "memory";
        let p2p_config = P2pConfig::new_with_memory_backend(server_name);
        let json_str = p2p_config.as_str();
        let p2p_config_2 = P2pConfig::from_str(&json_str).unwrap();
        assert_eq!(p2p_config, p2p_config_2);
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
