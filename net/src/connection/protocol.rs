//! This module provides the core low-level protocol enumeration
//! for communications between holochain core and the p2p / networking
//! process or library. See json_protocol for a higher level interface.

use serde_bytes;

use holochain_core_types::json::JsonString;

/// Low-level interface spec for communicating with the p2p abstraction
/// notice this is not Serializable or Deserializable
/// rmp_serde doesn't serialize enums very well... it uses indexes and arrays
/// which are not (easily) compatible with other endpoints
/// we use to/from NamedBinaryData to provide our own serialization wrapper
#[derive(Debug, Clone, PartialEq)]
pub enum Protocol {
    /// send/recv binary data / i.e. encryption, signature messages
    NamedBinary(NamedBinaryData),
    /// send/recv generic json as utf8 strings
    Json(JsonString),
    /// send/recv a Ping message (ipc protocol spec)
    Ping(PingData),
    /// send/recv a Pong message (ipc protocol spec)
    Pong(PongData),
    /// we have connected / configured the connection, ready for messages
    P2pReady,
}

/// provide utility for Protocol serialization
impl<'a> From<&'a Protocol> for NamedBinaryData {
    fn from(p: &'a Protocol) -> Self {
        match p {
            Protocol::NamedBinary(nb) => NamedBinaryData {
                name: b"namedBinary".to_vec(),
                data: rmp_serde::to_vec_named(nb).unwrap(),
            },
            Protocol::Json(j) => NamedBinaryData {
                name: b"json".to_vec(),
                data: String::from(j).into_bytes(),
            },
            Protocol::Ping(p) => NamedBinaryData {
                name: b"ping".to_vec(),
                data: rmp_serde::to_vec_named(p).unwrap(),
            },
            Protocol::Pong(p) => NamedBinaryData {
                name: b"pong".to_vec(),
                data: rmp_serde::to_vec_named(p).unwrap(),
            },
            Protocol::P2pReady => NamedBinaryData {
                name: b"p2pReady".to_vec(),
                data: Vec::new(),
            },
        }
    }
}

impl From<Protocol> for NamedBinaryData {
    fn from(p: Protocol) -> Self {
        (&p).into()
    }
}

/// provide utility for Protocol deserialization
impl<'a> From<&'a NamedBinaryData> for Protocol {
    fn from(nb: &'a NamedBinaryData) -> Self {
        match nb.name.as_slice() {
            b"namedBinary" => {
                let sub: NamedBinaryData = rmp_serde::from_slice(&nb.data).unwrap();
                Protocol::NamedBinary(sub)
            }
            b"json" => Protocol::Json(
                JsonString::from_json(
                    &String::from_utf8_lossy(&nb.data).to_string()
                )
            ),
            b"ping" => {
                let sub: PingData = rmp_serde::from_slice(&nb.data).unwrap();
                Protocol::Ping(sub)
            }
            b"pong" => {
                let sub: PongData = rmp_serde::from_slice(&nb.data).unwrap();
                Protocol::Pong(sub)
            }
            b"p2pReady" => Protocol::P2pReady,
            _ => panic!("bad Protocol type: {}", String::from_utf8_lossy(&nb.name)),
        }
    }
}

impl From<NamedBinaryData> for Protocol {
    fn from(nb: NamedBinaryData) -> Self {
        (&nb).into()
    }
}

impl<'a> From<&'a str> for Protocol {
    fn from(s: &'a str) -> Self {
        Protocol::Json(JsonString::from_json(&s.to_string()))
    }
}

impl From<String> for Protocol {
    fn from(s: String) -> Self {
        s.as_str().into()
    }
}

/// local macro for creating is_* and as_* functions on Protocol
/// (DRY some boilerplate)
macro_rules! simple_access {
    ($($is:ident $as:ident $d:ident $t:ty)*) => {
        $(
            pub fn $is(&self) -> bool {
                if let Protocol::$d(_) = self {
                    true
                } else {
                    false
                }
            }

            pub fn $as<'a>(&'a self) -> &'a $t {
                if let Protocol::$d(data) = self {
                    &data
                } else {
                    panic!(concat!(stringify!($as), " called with bad type"));
                }
            }
        )*
    }
}

impl Protocol {
    simple_access! {
        is_named_binary as_named_binary NamedBinary NamedBinaryData
        is_json as_json Json JsonString
        is_ping as_ping Ping PingData
        is_pong as_pong Pong PongData
    }

    /// get a json string straight out of the Protocol enum
    pub fn as_json_string(&self) -> JsonString {
        if let Protocol::Json(data) = self {
            JsonString::from_json(&String::from(data))
        } else {
            panic!("as_json_string called with bad type");
        }
    }
}

/// send/recv binary data / i.e. encryption, signature messages
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct NamedBinaryData {
    #[serde(with = "serde_bytes")]
    pub name: Vec<u8>,
    #[serde(with = "serde_bytes")]
    pub data: Vec<u8>,
}

/// send/recv a Ping message (ipc protocol spec)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PingData {
    pub sent: f64,
}

/// send/recv a Pong message (ipc protocol spec)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PongData {
    pub orig: f64,
    pub recv: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_handle_bad_type() {
        let p = Protocol::P2pReady;

        assert_eq!(false, p.is_json());
    }

    #[test]
    #[should_panic]
    fn it_should_panic_on_bad_as() {
        let p = Protocol::P2pReady;
        p.as_json();
    }

    #[test]
    #[should_panic]
    fn it_should_panic_on_bad_as_json_string() {
        let p = Protocol::P2pReady;
        p.as_json_string();
    }

    /// serialize and deserialize $e
    macro_rules! simple_convert {
        ($e:expr) => {{
            let wire: NamedBinaryData = $e.into();
            let res: Protocol = wire.into();
            res
        }};
    }

    #[test]
    fn it_can_convert_named_binary() {
        let nb_src = Protocol::NamedBinary(NamedBinaryData {
            name: b"test".to_vec(),
            data: b"hello".to_vec(),
        });

        let res = simple_convert!(nb_src);

        assert!(res.is_named_binary());

        let res = res.as_named_binary();

        assert_eq!(b"test".to_vec(), res.name);
        assert_eq!(b"hello".to_vec(), res.data);
    }

    #[test]
    fn it_can_convert_json() {
        let json_str = "{\"test\": \"hello\"}";
        let json: Protocol = json_str.to_string().into();

        let res = simple_convert!(json);

        assert!(res.is_json());

        let res = String::from(res.as_json_string());

        assert_eq!(json_str, res);
    }

    #[test]
    fn it_can_convert_ping() {
        let src = Protocol::Ping(PingData { sent: 42.0 });

        let res = simple_convert!(&src);

        assert!(res.is_ping());

        let res = res.as_ping();

        assert_eq!(42.0, res.sent);
    }

    #[test]
    fn it_can_convert_pong() {
        let src = Protocol::Pong(PongData {
            orig: 42.0,
            recv: 88.0,
        });

        let res = simple_convert!(&src);

        assert!(res.is_pong());

        let res = res.as_pong();

        assert_eq!(42.0, res.orig);
        assert_eq!(88.0, res.recv);
    }

    #[test]
    fn it_can_convert_p2p_ready() {
        let res = simple_convert!(&Protocol::P2pReady);

        assert_eq!(Protocol::P2pReady, res);
    }
}
