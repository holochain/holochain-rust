//! This module defines the trait for holochain networking / p2p
//! The goal is to make implementing concrete strucs as simple as possible
//! so that we can iterate quickly on the API.
//! All calls will go through one of two api functions, one using json
//! strings, and the other using binary buffers.

use base64;
use failure::Error;
use serde_json;

/// callback function type for api methods transporting json data
pub type ApiFnJson = Box<FnMut(&str) -> Result<String, Error>>;

/// callback function type for api methods transporting binary data
pub type ApiFnBin = Box<FnMut(&[u8]) -> Result<Vec<u8>, Error>>;

/// when the network is requesting we store data
/// this callback will be invoked, expecting the data to be validated
pub type DhtHoldCallback = Box<FnMut(&str) -> Result<bool, Error>>;

/// the identifier for an application
pub type GenomeHash = [u8; 32];

/// binary api function `track_app` uses this leading byte
pub const BIN_TYPE_TRACK_APP: u8 = 0x11;

/// binary api function `untrack_app` uses this leading byte
pub const BIN_TYPE_UNTRACK_APP: u8 = 0x12;

/// binary api function `set_app_signature_callback` uses this leading byte
pub const BIN_TYPE_APP_SIGNATURE: u8 = 0x21;

/// binary api function `set_app_encryption_callback` uses this leading byte
pub const BIN_TYPE_APP_ENCRYPTION: u8 = 0x22;

/// enum defining the state of this p2p/network connection
pub enum P2pNetworkState {
    /// we are still setting up the connection, please wait
    Pending,

    /// connection established, but needs config, please call `set_config`
    NeedConfig,

    /// connection established and configure, all APIs available
    Running,
}

/// Represents a connection to a peer to peer network module
/// On initial instantiation, only calling `get_state` is valid
/// see P2pNetorkState for usage
pub trait P2pNetwork {
    // -- only these top two functions are required to be implemented
    // -- by concrete structs

    /// This is the main backbone api throughput function
    /// that must be implemented by structs implementing this trait
    fn exec_raw_json(&mut self, input: &str, cb: Option<ApiFnJson>) -> Result<String, Error>;

    /// This is similar to `exec_raw_json`, but permits binary data transfer
    fn exec_raw_bin(&mut self, input: &[u8], cb: Option<ApiFnBin>) -> Result<Vec<u8>, Error>;

    // -- All following functions are default implementations
    // -- making use of the above two functions

    /// This call should return a state within:
    ///  - `pending`
    ///  - `need_config`
    ///  - `running`
    fn get_state(&mut self) -> Result<P2pNetworkState, Error> {
        let r = self.exec_raw_json(
            &(json!({
                "method": "getState"
            }).to_string()),
            None,
        )?;
        if "pending" == r {
            return Ok(P2pNetworkState::Pending);
        } else if "need_config" == r {
            return Ok(P2pNetworkState::NeedConfig);
        } else if "running" == r {
            return Ok(P2pNetworkState::Running);
        } else {
            bail!("unexpected state: '{}'", r);
        }
    }

    /// This call should return a json configuration blob for the p2p module
    fn get_default_config(&mut self) -> Result<String, Error> {
        self.exec_raw_json(
            &(json!({
                "method": "getDefaultConfig"
            }).to_string()),
            None,
        )
    }

    /// pass along configuration to the network module
    fn set_config(&mut self, config: &str) -> Result<(), Error> {
        let v: serde_json::value::Value = serde_json::from_str(config)?;
        let v = json!({
            "method": "setConfig",
            "config": v
        });
        self.exec_raw_json(&(v.to_string()), None)?;
        Ok(())
    }

    /// setup an app to be synced on the p2p network side
    fn track_app(
        &mut self,
        genome_hash: &GenomeHash,
        sign_pubkey: &[u8; 32],
        enc_pubkey: &[u8; 32],
    ) -> Result<(), Error> {
        let mut out: Vec<u8> = vec![BIN_TYPE_TRACK_APP];
        out.append(&mut genome_hash.to_vec());
        out.append(&mut sign_pubkey.to_vec());
        out.append(&mut enc_pubkey.to_vec());
        self.exec_raw_bin(out.as_slice(), None)?;
        Ok(())
    }

    /// stop syncing an app on the p2p network side
    fn untrack_app(&mut self, genome_hash: &GenomeHash) -> Result<(), Error> {
        let mut out: Vec<u8> = vec![BIN_TYPE_UNTRACK_APP];
        out.append(&mut genome_hash.to_vec());
        self.exec_raw_bin(out.as_slice(), None)?;
        Ok(())
    }

    /// set a signature callback for an app
    fn set_app_signature_callback(
        &mut self,
        genome_hash: &GenomeHash,
        cb: ApiFnBin,
    ) -> Result<(), Error> {
        let mut out: Vec<u8> = vec![BIN_TYPE_APP_SIGNATURE];
        out.append(&mut genome_hash.to_vec());
        self.exec_raw_bin(out.as_slice(), Some(cb))?;
        Ok(())
    }

    /// set an encryption callback for an app
    fn set_app_encryption_callback(
        &mut self,
        genome_hash: &GenomeHash,
        cb: ApiFnBin,
    ) -> Result<(), Error> {
        let mut out: Vec<u8> = vec![BIN_TYPE_APP_ENCRYPTION];
        out.append(&mut genome_hash.to_vec());
        self.exec_raw_bin(out.as_slice(), Some(cb))?;
        Ok(())
    }

    /// when the network asks us to store a bit of DHT data
    /// we first need to make sure it is valid
    fn dht_set_on_hold_callback(&mut self, mut cb: DhtHoldCallback) -> Result<(), Error> {
        self.exec_raw_json(
            &(json!({
                "method": "dhtOnHoldCallback"
            }).to_string()),
            Some(Box::new(move |input| {
                if cb(input)? {
                    return Ok("true".to_string());
                }
                return Ok("false".to_string());
            })),
        )?;
        Ok(())
    }

    /// we want to publish a bit of DHT data
    fn dht_publish(&mut self, genome_hash: &GenomeHash, data: &str) -> Result<(), Error> {
        let v: serde_json::value::Value = serde_json::from_str(data)?;
        let v = json!({
            "method": "dhtPublish",
            "genomeHash": base64::encode(genome_hash),
            "payload": v
        });
        self.exec_raw_json(&(v.to_string()), None)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// helper for easy conversion of None to an error type
    #[derive(Debug, Clone, Fail)]
    enum E {
        #[fail(display = "None")]
        None,
    }

    /// short name better than `Value`
    type Json = serde_json::value::Value;

    /// helper to convert a String into `Json`
    fn json_parse(input: &str) -> Result<Json, Error> {
        let v: Json = serde_json::from_str(input)?;
        Ok(v)
    }

    /// helper to grab a string from a `Json` instance of type object
    fn json_obj_str(input: &Json, property: &str) -> Result<String, Error> {
        Ok(input
            .as_object()
            .ok_or(E::None)?
            .get(property)
            .ok_or(E::None)?
            .as_str()
            .ok_or(E::None)?
            .to_string())
    }

    /// test struct stub function for handling ApiFnJson requests
    type JsonHandler = Box<FnMut(&str, Option<ApiFnJson>) -> Result<String, Error>>;

    /// test struct stub function for handling ApiFnBin requests
    type BinHandler = Box<FnMut(&[u8], Option<ApiFnBin>) -> Result<Vec<u8>, Error>>;

    /// test struct stub that will implement P2pNetwork trait
    struct P2pStub {
        pub json_handler_queue: Vec<JsonHandler>,
        pub bin_handler_queue: Vec<BinHandler>,
    }

    /// some custom test functions
    impl P2pStub {
        /// create a new network stub
        pub fn new() -> Self {
            P2pStub {
                json_handler_queue: Vec::new(),
                bin_handler_queue: Vec::new(),
            }
        }
    }

    /// the stub implementation will just sequentially execute any
    /// handlers that have been define in the test function
    impl P2pNetwork for P2pStub {
        /// execute the next test stub json handler
        fn exec_raw_json(&mut self, input: &str, cb: Option<ApiFnJson>) -> Result<String, Error> {
            self.json_handler_queue.remove(0)(input, cb)
        }

        /// execute the next test stub binary handler
        fn exec_raw_bin(&mut self, input: &[u8], cb: Option<ApiFnBin>) -> Result<Vec<u8>, Error> {
            self.bin_handler_queue.remove(0)(input, cb)
        }
    }

    /// a wrapper struct in case we need to track additional state
    struct NodeStub {
        pub net: P2pStub,
    }

    impl NodeStub {
        /// create a new wrapper struct
        pub fn new() -> Self {
            NodeStub {
                net: P2pStub::new(),
            }
        }
    }

    /// assert that an expression is None
    macro_rules! assert_none {
        ($e:expr) => {
            if let Some(_) = $e {
                panic!("was not None");
            }
        };
    }

    /// assert that an expression is Some(??)
    macro_rules! assert_some {
        ($e:expr) => {
            if let None = $e {
                panic!("was None, expected Some");
            }
        };
    }

    /// parse a json string and make sure it is an object
    /// with a "method" property that matches `$method`
    macro_rules! setup_handler {
        ($input:expr, $method:expr) => {{
            let v: Json = json_parse($input)?;
            assert_eq!($method.to_string(), json_obj_str(&v, "method")?);
            v
        }};
    }

    #[test]
    fn it_should_construct() {
        NodeStub::new();
    }

    #[test]
    fn it_should_return_default_config() {
        let mut node = NodeStub::new();
        node.net.json_handler_queue.push(Box::new(|input, cb| {
            assert_none!(cb);
            setup_handler!(input, "getDefaultConfig");
            Ok("{\"test\":\"holo\"}".to_string())
        }));
        assert_eq!(
            "{\"test\":\"holo\"}".to_string(),
            node.net.get_default_config().unwrap()
        );
    }

    #[test]
    fn it_should_return_state_pending() {
        let mut node = NodeStub::new();
        node.net.json_handler_queue.push(Box::new(|input, cb| {
            assert_none!(cb);
            setup_handler!(input, "getState");
            Ok("pending".to_string())
        }));
        match node.net.get_state().unwrap() {
            P2pNetworkState::Pending => (),
            _ => panic!("unexpected get_state return value"),
        };
    }

    #[test]
    fn it_should_return_state_need_config() {
        let mut node = NodeStub::new();
        node.net.json_handler_queue.push(Box::new(|input, cb| {
            assert_none!(cb);
            setup_handler!(input, "getState");
            Ok("need_config".to_string())
        }));
        match node.net.get_state().unwrap() {
            P2pNetworkState::NeedConfig => (),
            _ => panic!("unexpected get_state return value"),
        };
    }

    #[test]
    fn it_should_return_state_running() {
        let mut node = NodeStub::new();
        node.net.json_handler_queue.push(Box::new(|input, cb| {
            assert_none!(cb);
            setup_handler!(input, "getState");
            Ok("running".to_string())
        }));
        match node.net.get_state().unwrap() {
            P2pNetworkState::Running => (),
            _ => panic!("unexpected get_state return value"),
        };
    }

    #[test]
    fn it_should_set_config() {
        let mut node = NodeStub::new();
        node.net.json_handler_queue.push(Box::new(|input, cb| {
            assert_none!(cb);
            let v = setup_handler!(input, "setConfig");
            let c = v
                .as_object()
                .ok_or(E::None)?
                .get("config")
                .ok_or(E::None)?
                .to_string();
            assert_eq!("{\"test\":\"holo\"}".to_string(), c);
            Ok("undefined".to_string())
        }));
        node.net.set_config("{\"test\":\"holo\"}").unwrap();
    }

    #[test]
    fn it_should_track_app() {
        let mut node = NodeStub::new();
        node.net.bin_handler_queue.push(Box::new(|input, cb| {
            assert_none!(cb);
            assert_eq!(BIN_TYPE_TRACK_APP, input[0]);
            assert_eq!(1_u8, input[1]);
            assert_eq!(2_u8, input[33]);
            assert_eq!(3_u8, input[65]);
            Ok(Vec::new())
        }));
        node.net
            .track_app(&[1_u8; 32], &[2_u8; 32], &[3_u8; 32])
            .unwrap();
    }

    #[test]
    fn it_should_untrack_app() {
        let mut node = NodeStub::new();
        node.net.bin_handler_queue.push(Box::new(|input, cb| {
            assert_none!(cb);
            assert_eq!(BIN_TYPE_UNTRACK_APP, input[0]);
            assert_eq!(4_u8, input[1]);
            Ok(Vec::new())
        }));
        node.net.untrack_app(&[4_u8; 32]).unwrap();
    }

    #[test]
    fn it_should_set_app_sig_callback() {
        let mut node = NodeStub::new();
        node.net.bin_handler_queue.push(Box::new(|input, cb| {
            assert_some!(cb);
            assert_eq!(BIN_TYPE_APP_SIGNATURE, input[0]);
            assert_eq!(5_u8, input[1]);
            Ok(Vec::new())
        }));
        node.net
            .set_app_signature_callback(&[5_u8; 32], Box::new(|_i| Ok(Vec::new())))
            .unwrap();
    }

    #[test]
    fn it_should_set_app_enc_callback() {
        let mut node = NodeStub::new();
        node.net.bin_handler_queue.push(Box::new(|input, cb| {
            assert_some!(cb);
            assert_eq!(BIN_TYPE_APP_ENCRYPTION, input[0]);
            assert_eq!(6_u8, input[1]);
            Ok(Vec::new())
        }));
        node.net
            .set_app_encryption_callback(&[6_u8; 32], Box::new(|_i| Ok(Vec::new())))
            .unwrap();
    }

    #[test]
    fn it_should_call_on_dht_hold_callback_true() {
        let mut node = NodeStub::new();
        node.net.json_handler_queue.push(Box::new(|input, cb| {
            assert_some!(cb);
            setup_handler!(input, "dhtOnHoldCallback");
            let res = cb.unwrap()(
                &(json!({
                "genomeHash": "blabla",
                "payload": "blabla",
            }).to_string()),
            )?;
            assert_eq!("true".to_string(), res);
            Ok("undefined".to_string())
        }));
        node.net
            .dht_set_on_hold_callback(Box::new(|_i| Ok(true)))
            .unwrap();
    }

    #[test]
    fn it_should_call_on_dht_hold_callback_false() {
        let mut node = NodeStub::new();
        node.net.json_handler_queue.push(Box::new(|input, cb| {
            assert_some!(cb);
            setup_handler!(input, "dhtOnHoldCallback");
            let res = cb.unwrap()(
                &(json!({
                "genomeHash": "blabla",
                "payload": "blabla",
            }).to_string()),
            )?;
            assert_eq!("false".to_string(), res);
            Ok("undefined".to_string())
        }));
        node.net
            .dht_set_on_hold_callback(Box::new(|_i| Ok(false)))
            .unwrap();
    }

    #[test]
    fn it_should_dht_publish() {
        let mut node = NodeStub::new();
        node.net.json_handler_queue.push(Box::new(|input, cb| {
            assert_none!(cb);
            let v = setup_handler!(input, "dhtPublish");
            assert_eq!(
                "CQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQk=".to_string(),
                json_obj_str(&v, "genomeHash").unwrap()
            );
            let c = v
                .as_object()
                .ok_or(E::None)?
                .get("payload")
                .ok_or(E::None)?
                .to_string();
            assert_eq!("{\"test\":\"holo\"}".to_string(), c);
            Ok("undefined".to_string())
        }));
        node.net
            .dht_publish(&[9_u8; 32], "{\"test\":\"holo\"}")
            .unwrap();
    }
}
