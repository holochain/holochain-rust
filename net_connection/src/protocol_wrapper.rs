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

    #[serde(rename = "fromAgentId")]
    pub from_agent_id: String,

    pub address: String,
    pub attribute: String,
    pub content: serde_json::Value,
}

/// Enum holding all Message types in the 'hc-core <-> P2P network module' protocol.
/// There are 4 categories of messages:
///  - Command: An order from the local node to the p2p module. Local node expects a reponse. Starts with a verb.
///  - Handle-command: An order from the p2p module to the local node. The p2p module expects a response. Start withs 'Handle' followed by a verb.
///  - Result: A response to a Command. Starts with the name of the Command it responds to and ends with 'Result'.
///  - Notification: Notify that something happened. Not expecting any response. Ends with verb in past form, i.e. '-ed'.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DefaultJson)]
#[serde(tag = "method")]
pub enum ProtocolMessage {
    /// Request the current state from the p2p module
    #[serde(rename = "requestState")]
    GetState,
    /// p2p module's current state response.
    #[serde(rename = "state")]
    GetStateResult(StateData),
    /// Request the default config from the p2p module
    #[serde(rename = "requestDefaultConfig")]
    GetDefaultConfig,
    /// the p2p module's default config response
    #[serde(rename = "defaultConfig")]
    GetDefaultConfigResult(ConfigData),
    /// Set the p2p config
    #[serde(rename = "setConfig")]
    SetConfig(ConfigData),

    /// Connect to the specified multiaddr
    #[serde(rename = "connect")]
    Connect(ConnectData),
    /// Notification of a connection from another peer.
    #[serde(rename = "peerConnected")]
    PeerConnected(PeerData),

    /// Send a message to another peer on the network
    #[serde(rename = "send")]
    SendMessage(MessageData),
    /// the response from a previous `SendMessage`
    #[serde(rename = "sendResult")]
    SendMessageResult(MessageData),
    /// Request to handle a message another peer has sent us.
    #[serde(rename = "handleSend")]
    HandleSendMessage(MessageData),
    /// Our response to a message from another peer.
    #[serde(rename = "handleSendResult")]
    HandleSendMessageResult(MessageData),

    /// Order the p2p module to be part of the network of the specified DNA.
    #[serde(rename = "trackApp")]
    TrackDna(TrackAppData),

    /// Success response to any message with an _id field.
    #[serde(rename = "successResult")]
    SuccessResult(SuccessResultData),
    /// Failure response to any message with an _id field.
    /// Can also be a response to a mal-formed request.
    #[serde(rename = "failureResult")]
    FailureResult(FailureResultData),

    /// [send] request data from the dht
    /// [recv] another node, or the network module itself is requesting data
    ///        from us... send a GetDhtResult message back
    #[serde(rename = "getDht")]
    GetDhtData(GetDhtData),
    /// [recv] response from requesting dht data from the network
    /// [send] success response if network is requesting this data of us
    #[serde(rename = "getDhtResult")]
    GetDhtDataResult(DhtData),

    /// Publish data to the dht.
    #[serde(rename = "publishDht")]
    PublishDhtData(DhtData),
    /// Store data on a node's dht slice.
    #[serde(rename = "storeDht")]
    HandleStoreDhtData(DhtData),

    /// [send] request meta data from the dht
    /// [recv] another node, or the network module itself is requesting data
    ///        from us... send a GetDhtResult message back
    #[serde(rename = "getDhtMeta")]
    GetDhtMeta(GetDhtMetaData),
    /// [recv] response from requesting meta dht data from the network
    /// [send] success response if network is requesting this data of us
    #[serde(rename = "getDhtMetaResult")]
    GetDhtMetaResult(DhtMetaData),

    /// Publish metadata to the dht.
    #[serde(rename = "publishDhtMeta")]
    PublishDhtMeta(DhtMetaData),
    /// Store metadata on a node's dht slice.
    #[serde(rename = "storeDhtMeta")]
    HandleStoreDhtMeta(DhtMetaData),
}

impl<'a> TryFrom<&'a Protocol> for ProtocolMessage {
    type Error = Error;
    fn try_from(p: &Protocol) -> Result<Self, Error> {
        if let Protocol::Json(json) = p {
            match ProtocolMessage::try_from(json) {
                Ok(w) => {
                    return Ok(w);
                }
                Err(e) => bail!("{:?}", e),
            };
        }
        bail!("could not ProtocolWrapper: {:?}", p);
    }
}

impl TryFrom<Protocol> for ProtocolMessage {
    type Error = Error;
    fn try_from(p: Protocol) -> Result<Self, Error> {
        ProtocolMessage::try_from(&p)
    }
}

impl<'a> From<&'a ProtocolMessage> for Protocol {
    fn from(w: &ProtocolMessage) -> Self {
        Protocol::Json(JsonString::from(w))
    }
}

impl From<ProtocolMessage> for Protocol {
    fn from(w: ProtocolMessage) -> Self {
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
        let w = ProtocolMessage::try_from(JsonString::from(
            r#"{
            "method": "state",
            "state": "test_state"
        }"#,
        ))
        .unwrap();
        if let ProtocolMessage::GetStateResult(s) = w {
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
