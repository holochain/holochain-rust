// p2p_network.rs

use failure::Error;
use serde_json;

#[derive(Debug, Clone, Fail)]
pub enum E {
    #[fail(display = "None")]
    None,
}

pub type Json = serde_json::value::Value;

pub fn json_parse(input: &str) -> Result<Json, Error> {
    let v: Json = serde_json::from_str(input)?;
    Ok(v)
}

pub fn json_obj_str(input: &Json, property: &str) -> Result<String, Error> {
    Ok(input
        .as_object()
        .ok_or(E::None)?
        .get(property)
        .ok_or(E::None)?
        .as_str()
        .ok_or(E::None)?
        .to_string())
}

/// callback function type for json functions
pub type ApiFnJson = Box<FnMut(&str) -> Result<String, Error>>;

/// callback function type for binary functions
pub type ApiFnBin = Box<FnMut(&[u8]) -> Result<Vec<u8>, Error>>;

/// Represents a connection to a peer to peer network module
pub trait P2pNetwork {
    /// This is the main backbone api throughput function
    /// that must be implemented by structs implementing this trait
    fn exec_raw_json(&mut self, input: &str, cb: Option<ApiFnJson>) -> Result<String, Error>;

    /// This is similar to `exec_raw_json`, but permits binary data transfer
    fn exec_raw_bin(&mut self, input: &[u8], cb: Option<ApiFnBin>) -> Result<Vec<u8>, Error>;

    /// This call should return a json configuration blob for the p2p module
    fn get_default_config(&mut self) -> Result<String, Error> {
        self.exec_raw_json(
            &(json!({
            "method": "getDefaultConfig"
        }).to_string()),
            None,
        )
    }

    /*
    // example of calling a function in the network
    fn add(&mut self, a: f64, b: f64, mut cb: CbAddResult) {
        self.exec_raw_json(&(json!({
            "method": "add",
            "a": a,
            "b": b
        }).to_string()), Some(Box::new(move |result| {
            let v: serde_json::value::Value = serde_json::from_str(result)?;
            let v = v.as_object().unwrap();
            let v = v.get("result").unwrap().as_f64().unwrap();
            cb(v)?;
            Ok("undefined".to_string())
        }))).unwrap();
    }

    // example of the network fetching data from the core side
    fn call_me_subtract(&mut self, mut cb: CbDoSubtract) {
        self.exec_raw_json(&(json!({
            "method": "call-me-subtract"
        }).to_string()), Some(Box::new(move |data| {
            let v: serde_json::value::Value = serde_json::from_str(data)?;
            let v = v.as_object().unwrap();

            let a = v.get("a").unwrap().as_f64().unwrap();
            let b = v.get("b").unwrap().as_f64().unwrap();
            let c = cb(a, b)?;

            Ok(json!({
                "result": c
            }).to_string())
        }))).unwrap();
    }
    */
}

#[cfg(test)]
mod tests {
    use super::*;

    pub type JsonHandler = Box<FnMut(&str, Option<ApiFnJson>) -> Result<String, Error>>;

    pub struct P2pStub {
        pub json_handler_queue: Vec<JsonHandler>,
    }

    impl P2pStub {
        pub fn new() -> Self {
            P2pStub {
                json_handler_queue: Vec::new(),
            }
        }
    }

    impl P2pNetwork for P2pStub {
        fn exec_raw_json(&mut self, input: &str, cb: Option<ApiFnJson>) -> Result<String, Error> {
            self.json_handler_queue.remove(0)(input, cb)
        }

        fn exec_raw_bin(&mut self, _input: &[u8], _cb: Option<ApiFnBin>) -> Result<Vec<u8>, Error> {
            Ok([].to_vec())
        }
    }

    pub struct NodeStub {
        pub net: P2pStub,
    }

    impl NodeStub {
        pub fn new() -> Self {
            NodeStub {
                net: P2pStub::new(),
            }
        }
    }

    #[test]
    fn it_should_construct() {
        NodeStub::new();
    }

    #[test]
    fn it_should_return_default_config() {
        let mut node = NodeStub::new();
        node.net.json_handler_queue.push(Box::new(|input, cb| {
            if let Some(_) = cb {
                panic!("cb should be none");
            }
            let v: Json = json_parse(input)?;
            let v = json_obj_str(&v, "method")?;
            assert_eq!("getDefaultConfig".to_string(), v);
            Ok("{\"test\":\"holo\"}".to_string())
        }));
        assert_eq!(
            "{\"test\":\"holo\"}".to_string(),
            node.net.get_default_config().unwrap()
        );
    }
}
