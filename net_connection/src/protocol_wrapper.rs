//! This module provides a higher level interface to p2p / network messaging
//! basically handles serialization / deserialization from / to the core
//! protocol message types (NamedBinary and Json).

use serde_json;

use super::protocol::Protocol;

/// High level p2p / network message
#[derive(Debug, Clone, PartialEq)]
pub enum ProtocolWrapper {
    /// if we don't have a high-level repr, just store the base protocol
    RawProtocol(Protocol),

    /// [send] request the current state from the p2p module
    RequestState,

    /// [recv] p2p module is telling us the current state
    State(StateData),

    /// [send] request the default config from the p2p module
    RequestDefaultConfig,

    /// [recv] the default config from the p2p module
    DefaultConfig(ConfigData),

    /// [send] set the p2p config
    SetConfig(ConfigData),

    /// [send] connect to the specified multiaddr
    Connect(ConnectData),

    /// [recv] notification of a peer connected
    PeerConnected(PeerConnectData),

    /// [send] send a message to another node on the network
    SendMessage(SendData),

    /// [recv] recv the response back from a previous `SendMessage`
    SendResult(SendResultData),

    /// [recv] another node has sent us a message
    HandleSend(HandleSendData),

    /// [send] send our response to a previous `HandleSend`
    HandleSendResult(SendResultData),
}

/// upgrade a Protocol reference to a ProtocolWrapper instance
impl<'a> From<&'a Protocol> for ProtocolWrapper {
    fn from(p: &'a Protocol) -> Self {
        if let Protocol::Json(json) = p {
            let json: serde_json::Value = serde_json::from_str(json.into()).unwrap();
            let method = &json["method"];

            if method == "requestState" {
                return ProtocolWrapper::RequestState;
            } else if method == "state" {
                return ProtocolWrapper::State(StateData {
                    state: match json["state"].as_str() {
                        Some(s) => s.to_string(),
                        None => "undefined".to_string(),
                    },
                    id: match json["id"].as_str() {
                        Some(s) => s.to_string(),
                        None => "undefined".to_string(),
                    },
                    bindings: match json["bindings"].as_array() {
                        Some(arr) => arr
                            .iter()
                            .map(|i| match i.as_str() {
                                Some(b) => b.to_string(),
                                None => "undefined".to_string(),
                            })
                            .collect(),
                        None => Vec::new(),
                    },
                });
            } else if method == "requestDefaultConfig" {
                return ProtocolWrapper::RequestDefaultConfig;
            } else if method == "defaultConfig" {
                assert!(json["config"].is_string());
                return ProtocolWrapper::DefaultConfig(ConfigData {
                    config: json["config"].as_str().unwrap().to_string(),
                });
            } else if method == "setConfig" {
                assert!(json["config"].is_string());
                return ProtocolWrapper::SetConfig(ConfigData {
                    config: json["config"].as_str().unwrap().to_string(),
                });
            } else if method == "connect" {
                assert!(json["address"].is_string());
                return ProtocolWrapper::Connect(ConnectData {
                    address: json["address"].as_str().unwrap().to_string(),
                });
            } else if method == "peerConnected" {
                assert!(json["id"].is_string());
                return ProtocolWrapper::PeerConnected(PeerConnectData {
                    id: json["id"].as_str().unwrap().to_string(),
                });
            } else if method == "send" {
                assert!(json["_id"].is_string());
                assert!(json["toAddress"].is_string());
                return ProtocolWrapper::SendMessage(SendData {
                    msg_id: json["_id"].as_str().unwrap().to_string(),
                    to_address: json["toAddress"].as_str().unwrap().to_string(),
                    data: json["data"].clone(),
                });
            } else if method == "handleSend" {
                assert!(json["_id"].is_string());
                assert!(json["toAddress"].is_string());
                assert!(json["fromAddress"].is_string());
                return ProtocolWrapper::HandleSend(HandleSendData {
                    msg_id: json["_id"].as_str().unwrap().to_string(),
                    to_address: json["toAddress"].as_str().unwrap().to_string(),
                    from_address: json["fromAddress"].as_str().unwrap().to_string(),
                    data: json["data"].clone(),
                });
            } else if method == "handleSendResult" {
                assert!(json["_id"].is_string());
                return ProtocolWrapper::HandleSendResult(SendResultData {
                    msg_id: json["_id"].as_str().unwrap().to_string(),
                    data: json["data"].clone(),
                });
            } else if method == "sendResult" {
                assert!(json["_id"].is_string());
                return ProtocolWrapper::SendResult(SendResultData {
                    msg_id: json["_id"].as_str().unwrap().to_string(),
                    data: json["data"].clone(),
                });
            }
        }

        ProtocolWrapper::RawProtocol(p.clone())
    }
}

impl From<Protocol> for ProtocolWrapper {
    fn from(p: Protocol) -> Self {
        ProtocolWrapper::from(&p)
    }
}

/// downgrade a ProtocolWrapper instance back into a Protocol message
impl<'a> From<&'a ProtocolWrapper> for Protocol {
    // david.b (neonphog) - tarpaulin doesn't cover macros... skipping this fn
    #[cfg_attr(tarpaulin, skip)]
    fn from(w: &'a ProtocolWrapper) -> Self {
        match w {
            ProtocolWrapper::RawProtocol(p) => p.clone(),
            ProtocolWrapper::RequestState => Protocol::Json(
                json!({
                    "method": "requestState",
                }).into(),
            ),
            ProtocolWrapper::State(s) => Protocol::Json(
                json!({
                    "method": "state",
                    "state": s.state,
                    "id": s.id,
                    "bindings": s.bindings,
                }).into(),
            ),
            ProtocolWrapper::RequestDefaultConfig => Protocol::Json(
                json!({
                    "method": "requestDefaultConfig",
                }).into(),
            ),
            ProtocolWrapper::DefaultConfig(c) => Protocol::Json(
                json!({
                    "method": "defaultConfig",
                    "config": c.config,
                }).into(),
            ),
            ProtocolWrapper::SetConfig(c) => Protocol::Json(
                json!({
                    "method": "setConfig",
                    "config": c.config,
                }).into(),
            ),
            ProtocolWrapper::Connect(c) => Protocol::Json(
                json!({
                    "method": "connect",
                    "address": c.address,
                }).into(),
            ),
            ProtocolWrapper::PeerConnected(c) => Protocol::Json(
                json!({
                    "method": "peerConnected",
                    "id": c.id,
                }).into(),
            ),
            ProtocolWrapper::SendMessage(m) => Protocol::Json(
                json!({
                    "method": "send",
                    "_id": m.msg_id,
                    "toAddress": m.to_address,
                    "data": m.data,
                }).into(),
            ),
            ProtocolWrapper::HandleSend(m) => Protocol::Json(
                json!({
                    "method": "handleSend",
                    "_id": m.msg_id,
                    "toAddress": m.to_address,
                    "fromAddress": m.from_address,
                    "data": m.data,
                }).into(),
            ),
            ProtocolWrapper::HandleSendResult(m) => Protocol::Json(
                json!({
                    "method": "handleSendResult",
                    "_id": m.msg_id,
                    "data": m.data,
                }).into(),
            ),
            ProtocolWrapper::SendResult(m) => Protocol::Json(
                json!({
                    "method": "sendResult",
                    "_id": m.msg_id,
                    "data": m.data,
                }).into(),
            ),
        }
    }
}

impl From<ProtocolWrapper> for Protocol {
    fn from(w: ProtocolWrapper) -> Self {
        Protocol::from(&w)
    }
}

/// state of the p2p connection
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct StateData {
    /// 'need_config' / 'pending' / 'ready'
    pub state: String,
    /// the transport identifier of this node
    pub id: String,
    /// the opaque transport bindings for bootstrapping others
    pub bindings: Vec<String>,
}

/// configuration info
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ConfigData {
    /// should be serialized in a human-readable format (e.g. json, toml, etc)
    pub config: String,
}

/// multiaddr to connect to
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ConnectData {
    pub address: String,
}

/// node connection info
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PeerConnectData {
    pub id: String,
}

/// send a message to another node
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SendData {
    /// the `SendResultData` will contain the id you put here
    pub msg_id: String,
    /// the id of the node to message
    pub to_address: String,
    /// the data to send
    pub data: serde_json::Value,
}

/// we need to handle a `send` from another node
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct HandleSendData {
    /// when you pass back the `HandleSendResult`, use this id
    pub msg_id: String,
    /// this should be your id
    pub to_address: String,
    /// the id of the node sending this message
    pub from_address: String,
    /// the data they are sending
    pub data: serde_json::Value,
}

/// when processing a SendResult or a HandleSendResult
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SendResultData {
    /// the id associated with this message
    pub msg_id: String,
    /// the data associated with this response
    pub data: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_convert_request_state() {
        let p: Protocol = ProtocolWrapper::RequestState.into();
        assert_eq!("{\"method\":\"requestState\"}", &p.as_json_string());
        let w: ProtocolWrapper = p.into();
        assert_eq!(ProtocolWrapper::RequestState, w);
    }

    #[test]
    fn it_can_convert_state() {
        let orig = ProtocolWrapper::State(StateData {
            state: "test_state".to_string(),
            id: "test_id".to_string(),
            bindings: vec!["test_b1".to_string(), "test_b2".to_string()],
        });
        let p: Protocol = (&orig).into();
        let w: ProtocolWrapper = p.into();
        assert_eq!(orig, w);
    }

    #[test]
    fn it_can_convert_weird_state() {
        let w: ProtocolWrapper = Protocol::Json(
            json!({
            "method": "state",
            "bindings": [null],
        }).into(),
        ).into();
        assert_eq!(
            w,
            ProtocolWrapper::State(StateData {
                state: "undefined".to_string(),
                id: "undefined".to_string(),
                bindings: vec!["undefined".to_string()],
            })
        );
    }

    #[test]
    fn it_can_convert_request_default_config() {
        let p: Protocol = ProtocolWrapper::RequestDefaultConfig.into();
        let w: ProtocolWrapper = p.into();
        assert_eq!(ProtocolWrapper::RequestDefaultConfig, w);
    }

    #[test]
    fn it_can_convert_default_config() {
        let orig = ProtocolWrapper::DefaultConfig(ConfigData {
            config: "test_config".to_string(),
        });
        let p: Protocol = (&orig).into();
        let w: ProtocolWrapper = p.into();
        assert_eq!(orig, w);
    }

    #[test]
    fn it_can_convert_set_config() {
        let orig = ProtocolWrapper::SetConfig(ConfigData {
            config: "test_config".to_string(),
        });
        let p: Protocol = (&orig).into();
        let w: ProtocolWrapper = p.into();
        assert_eq!(orig, w);
    }

    #[test]
    fn it_can_convert_connect() {
        let orig = ProtocolWrapper::Connect(ConnectData {
            address: "test_addr".to_string(),
        });
        let p: Protocol = (&orig).into();
        let w: ProtocolWrapper = p.into();
        assert_eq!(orig, w);
    }

    #[test]
    fn it_can_convert_peer_connected() {
        let orig = ProtocolWrapper::PeerConnected(PeerConnectData {
            id: "test_id".to_string(),
        });
        let p: Protocol = (&orig).into();
        let w: ProtocolWrapper = p.into();
        assert_eq!(orig, w);
    }

    #[test]
    fn it_can_convert_send_message() {
        let orig = ProtocolWrapper::SendMessage(SendData {
            msg_id: "test_id".to_string(),
            to_address: "test_addr".to_string(),
            data: "test_data".into(),
        });
        let p: Protocol = (&orig).into();
        let w: ProtocolWrapper = p.into();
        assert_eq!(orig, w);
    }

    #[test]
    fn it_can_convert_send_result() {
        let orig = ProtocolWrapper::SendResult(SendResultData {
            msg_id: "test_id".to_string(),
            data: "test_data".into(),
        });
        let p: Protocol = (&orig).into();
        let w: ProtocolWrapper = p.into();
        assert_eq!(orig, w);
    }

    #[test]
    fn it_can_convert_handle_send() {
        let orig = ProtocolWrapper::HandleSend(HandleSendData {
            msg_id: "test_id".to_string(),
            to_address: "test_addr".to_string(),
            from_address: "test_addr2".to_string(),
            data: "test_data".into(),
        });
        let p: Protocol = (&orig).into();
        let w: ProtocolWrapper = p.into();
        assert_eq!(orig, w);
    }

    #[test]
    fn it_can_convert_handle_send_result() {
        let orig = ProtocolWrapper::HandleSendResult(SendResultData {
            msg_id: "test_id".to_string(),
            data: "test_data".into(),
        });
        let p: Protocol = (&orig).into();
        let w: ProtocolWrapper = p.into();
        assert_eq!(orig, w);
    }

}
