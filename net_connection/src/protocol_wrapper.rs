//! This module provides a higher level interface to p2p / network messaging
//! basically handles serialization / deserialization from / to the core
//! protocol message types (NamedBinary and Json).

use serde_json;

use failure::Error;
use holochain_core_types::{error::HolochainError, json::JsonString};
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
    pub address: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DefaultJson)]
pub struct PeerData {
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DefaultJson)]
pub struct SendMessageData {
    #[serde(rename = "_id")]
    pub msg_id: String,

    #[serde(rename = "toAddress")]
    pub to_address: String,

    pub data: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DefaultJson)]
pub struct SendResultData {
    #[serde(rename = "_id")]
    pub msg_id: String,

    pub data: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DefaultJson)]
pub struct HandleSendData {
    #[serde(rename = "_id")]
    pub msg_id: String,

    #[serde(rename = "toAddress")]
    pub to_address: String,

    #[serde(rename = "fromAddress")]
    pub from_address: String,

    pub data: serde_json::Value,
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
    SendMessage(SendMessageData),

    /// [recv] recv the response back from a previous `SendMessage`
    #[serde(rename = "sendResult")]
    SendResult(SendResultData),

    /// [recv] another node has sent us a message
    #[serde(rename = "handleSend")]
    HandleSend(HandleSendData),

    /// [send] send our response to a previous `HandleSend`
    #[serde(rename = "handleSendResult")]
    HandleSendResult(SendResultData),
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
        )).unwrap();
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
            address: "test".to_string(),
        }));
    }

    #[test]
    fn it_can_convert_peer_connected() {
        test_convert!(ProtocolWrapper::PeerConnected(PeerData {
            id: "test".to_string(),
        }));
    }

    #[test]
    fn it_can_convert_send_message() {
        test_convert!(ProtocolWrapper::SendMessage(SendMessageData {
            msg_id: "test_id".to_string(),
            to_address: "test_addr".to_string(),
            data: json!("hello"),
        }));
    }

    #[test]
    fn it_can_convert_send_result() {
        test_convert!(ProtocolWrapper::SendResult(SendResultData {
            msg_id: "test_id".to_string(),
            data: json!("hello"),
        }));
    }

    #[test]
    fn it_can_convert_handle_send() {
        test_convert!(ProtocolWrapper::HandleSend(HandleSendData {
            msg_id: "test_id".to_string(),
            to_address: "test_addr".to_string(),
            from_address: "test_addr2".to_string(),
            data: json!("hello"),
        }));
    }

    #[test]
    fn it_can_convert_handle_send_result() {
        test_convert!(ProtocolWrapper::HandleSendResult(SendResultData {
            msg_id: "test_id".to_string(),
            data: json!("hello"),
        }));
    }

}
