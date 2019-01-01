//! This module provides a higher level interface to p2p / network messaging
//! basically handles serialization / deserialization from / to the core
//! protocol message types (NamedBinary and Json).

use serde_json;

use failure::Error;
use holochain_core_types::{cas::content::Address, error::HolochainError, json::JsonString};
use std::convert::TryFrom;

use super::protocol::Protocol;

fn get_default_state_id() -> String {
    "undefined".to_string()
}

fn get_default_state_bindings() -> Vec<String> {
    Vec::new()
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DefaultJson)]
pub struct StateData {
    pub state: String,
    #[serde(default = "get_default_state_id")]
    pub id: String,
    #[serde(default = "get_default_state_bindings")]
    pub bindings: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DefaultJson)]
pub struct ConfigData {
    pub config: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DefaultJson)]
pub struct ConnectData {
    pub address: Address,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DefaultJson)]
pub struct PeerData {
    #[serde(rename = "dnaAddress")]
    pub dna_address: Address,

    #[serde(rename = "agentId")]
    pub agent_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DefaultJson)]
pub struct MessageData {
    #[serde(rename = "_id")]
    pub msg_id: String,

    #[serde(rename = "dnaAddress")]
    pub dna_address: Address,

    #[serde(rename = "toAgentId")]
    pub to_agent_id: String,

    #[serde(rename = "fromAgentId")]
    pub from_agent_id: String,

    pub data: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DefaultJson)]
pub struct TrackAppData {
    #[serde(rename = "dnaAddress")]
    pub dna_address: Address,

    #[serde(rename = "agentId")]
    pub agent_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DefaultJson)]
pub struct SuccessResultData {
    #[serde(rename = "_id")]
    pub msg_id: String,

    #[serde(rename = "dnaAddress")]
    pub dna_address: Address,

    #[serde(rename = "toAgentId")]
    pub to_agent_id: String,

    #[serde(rename = "successInfo")]
    pub success_info: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DefaultJson)]
pub struct FailureResultData {
    #[serde(rename = "_id")]
    pub msg_id: String,

    #[serde(rename = "dnaAddress")]
    pub dna_address: Address,

    #[serde(rename = "toAgentId")]
    pub to_agent_id: String,

    #[serde(rename = "errorInfo")]
    pub error_info: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DefaultJson)]
pub struct GetDhtData {
    #[serde(rename = "_id")]
    pub msg_id: String,

    #[serde(rename = "dnaAddress")]
    pub dna_address: Address,

    #[serde(rename = "fromAgentId")]
    pub from_agent_id: String,

    pub address: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DefaultJson, Default)]
pub struct DhtData {
    #[serde(rename = "_id")]
    pub msg_id: String,

    #[serde(rename = "dnaAddress")]
    pub dna_address: Address,

    #[serde(rename = "agentId")]
    pub agent_id: String,

    pub address: String,
    pub content: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DefaultJson)]
pub struct GetDhtMetaData {
    #[serde(rename = "_id")]
    pub msg_id: String,

    #[serde(rename = "dnaAddress")]
    pub dna_address: Address,

    #[serde(rename = "fromAgentId")]
    pub from_agent_id: String,

    pub address: String,
    pub attribute: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DefaultJson)]
pub struct DhtMetaData {
    #[serde(rename = "_id")]
    pub msg_id: String,

    #[serde(rename = "dnaAddress")]
    pub dna_address: Address,

    #[serde(rename = "agentId")]
    pub agent_id: String,

    pub address: String,
    pub attribute: String,
    pub content: serde_json::Value,
}

/// High level p2p / network message
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DefaultJson)]
#[serde(tag = "method")]
pub enum ProtocolWrapper {
    /// [send] request the current state from the p2p module
    #[serde(rename = "requestState")]
    RequestState,

    /// [recv] p2p module is telling us the current state
    #[serde(rename = "state")]
    State(StateData),

    /// [send] request the default config from the p2p module
    #[serde(rename = "requestDefaultConfig")]
    RequestDefaultConfig,

    /// [recv] the default config from the p2p module
    #[serde(rename = "defaultConfig")]
    DefaultConfig(ConfigData),

    /// [send] set the p2p config
    #[serde(rename = "setConfig")]
    SetConfig(ConfigData),

    /// [send] connect to the specified multiaddr
    #[serde(rename = "connect")]
    Connect(ConnectData),

    /// [recv] notification of a peer connected
    #[serde(rename = "peerConnected")]
    PeerConnected(PeerData),

    /// [send] send a message to another node on the network
    #[serde(rename = "send")]
    SendMessage(MessageData),

    /// [recv] recv the response back from a previous `SendMessage`
    #[serde(rename = "sendResult")]
    SendResult(MessageData),

    /// [recv] another node has sent us a message
    #[serde(rename = "handleSend")]
    HandleSend(MessageData),

    /// [send] send our response to a previous `HandleSend`
    #[serde(rename = "handleSendResult")]
    HandleSendResult(MessageData),

    /// [send] send out a "trackApp" request
    #[serde(rename = "trackApp")]
    TrackApp(TrackAppData),

    /// [send / recv] report success for a messages with _id parameter
    #[serde(rename = "successResult")]
    SuccessResult(SuccessResultData),

    /// [send / recv] for any message with _id parameter to indicate failure
    #[serde(rename = "failureResult")]
    FailureResult(FailureResultData),

    /// [send] request data from the dht
    /// [recv] another node, or the network module itself is requesting data
    ///        from us... send a GetDhtResult message back
    #[serde(rename = "getDht")]
    GetDht(GetDhtData),

    /// [recv] response from requesting dht data from the network
    /// [send] success response if network is requesting this data of us
    #[serde(rename = "getDhtResult")]
    GetDhtResult(DhtData),

    /// [send] publish content to the dht
    #[serde(rename = "publishDht")]
    PublishDht(DhtData),

    /// [recv] the network is requesting that we store this data
    #[serde(rename = "storeDht")]
    StoreDht(DhtData),

    /// [send] request meta data from the dht
    /// [recv] another node, or the network module itself is requesting data
    ///        from us... send a GetDhtResult message back
    #[serde(rename = "getDhtMeta")]
    GetDhtMeta(GetDhtMetaData),

    /// [recv] response from requesting meta dht data from the network
    /// [send] success response if network is requesting this data of us
    #[serde(rename = "getDhtMetaResult")]
    GetDhtMetaResult(DhtMetaData),

    /// [send] publish meta content to the dht
    #[serde(rename = "publishDhtMeta")]
    PublishDhtMeta(DhtMetaData),

    /// [recv] the network is requesting that we store this meta data
    #[serde(rename = "storeDhtMeta")]
    StoreDhtMeta(DhtMetaData),
}

impl<'a> TryFrom<&'a Protocol> for ProtocolWrapper {
    type Error = Error;
    fn try_from(p: &Protocol) -> Result<Self, Error> {
        if let Protocol::Json(json) = p {
            match ProtocolWrapper::try_from(json) {
                Ok(w) => {
                    return Ok(w);
                }
                Err(e) => bail!("{:?}", e),
            };
        }
        bail!("could not ProtocolWrapper: {:?}", p);
    }
}

impl TryFrom<Protocol> for ProtocolWrapper {
    type Error = Error;
    fn try_from(p: Protocol) -> Result<Self, Error> {
        ProtocolWrapper::try_from(&p)
    }
}

impl<'a> From<&'a ProtocolWrapper> for Protocol {
    fn from(w: &ProtocolWrapper) -> Self {
        Protocol::Json(JsonString::from(w))
    }
}

impl From<ProtocolWrapper> for Protocol {
    fn from(w: ProtocolWrapper) -> Self {
        Protocol::from(&w)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_convert {
        ($e:expr) => {
            let orig = $e;
            let p = Protocol::from(orig.clone());
            let w = ProtocolWrapper::try_from(p).unwrap();
            assert_eq!(orig, w);
        };
    }

    #[test]
    fn it_can_convert_request_state() {
        test_convert!(ProtocolWrapper::RequestState);
    }

    #[test]
    fn it_can_convert_state() {
        test_convert!(ProtocolWrapper::State(StateData {
            state: "test_state".to_string(),
            id: "test_id".to_string(),
            bindings: vec!["test_binding".to_string()],
        }));
    }

    #[test]
    fn it_can_convert_funky_state() {
        let w = ProtocolWrapper::try_from(JsonString::from(
            r#"{
            "method": "state",
            "state": "test_state"
        }"#,
        ))
        .unwrap();
        if let ProtocolWrapper::State(s) = w {
            assert_eq!("undefined", &s.id);
            assert_eq!(0, s.bindings.len());
        } else {
            panic!("bad enum type");
        }
    }

    #[test]
    fn it_can_convert_request_default_config() {
        test_convert!(ProtocolWrapper::RequestDefaultConfig);
    }

    #[test]
    fn it_can_convert_default_config() {
        test_convert!(ProtocolWrapper::DefaultConfig(ConfigData {
            config: "test".to_string(),
        }));
    }

    #[test]
    fn it_can_convert_set_config() {
        test_convert!(ProtocolWrapper::SetConfig(ConfigData {
            config: "test".to_string(),
        }));
    }

    #[test]
    fn it_can_convert_set_connect() {
        test_convert!(ProtocolWrapper::Connect(ConnectData {
            address: "test".into(),
        }));
    }

    #[test]
    fn it_can_convert_peer_connected() {
        test_convert!(ProtocolWrapper::PeerConnected(PeerData {
            dna_address: "test_dna".into(),
            agent_id: "test_id".to_string(),
        }));
    }

    #[test]
    fn it_can_convert_send_message() {
        test_convert!(ProtocolWrapper::SendMessage(MessageData {
            dna_address: "test_dna".into(),
            to_agent_id: "test_to".to_string(),
            from_agent_id: "test_from".to_string(),
            msg_id: "test_id".to_string(),
            data: json!("hello"),
        }));
    }

    #[test]
    fn it_can_convert_send_result() {
        test_convert!(ProtocolWrapper::SendResult(MessageData {
            dna_address: "test_dna".into(),
            to_agent_id: "test_to".to_string(),
            from_agent_id: "test_from".to_string(),
            msg_id: "test_id".to_string(),
            data: json!("hello"),
        }));
    }

    #[test]
    fn it_can_convert_handle_send() {
        test_convert!(ProtocolWrapper::HandleSend(MessageData {
            dna_address: "test_dna".into(),
            to_agent_id: "test_to".to_string(),
            from_agent_id: "test_from".to_string(),
            msg_id: "test_id".to_string(),
            data: json!("hello"),
        }));
    }

    #[test]
    fn it_can_convert_handle_send_result() {
        test_convert!(ProtocolWrapper::HandleSendResult(MessageData {
            dna_address: "test_dna".into(),
            to_agent_id: "test_to".to_string(),
            from_agent_id: "test_from".to_string(),
            msg_id: "test_id".to_string(),
            data: json!("hello"),
        }));
    }

    #[test]
    fn it_can_convert_track_app() {
        test_convert!(ProtocolWrapper::TrackApp(TrackAppData {
            dna_address: "test_dna".into(),
            agent_id: "test_to".to_string(),
        }));
    }
}
