//! holochain_net is a library that defines an abstract networking layer for
//! different network transports and implements a PeerStore for mapping and
//! managing the topology of transport layers with regard to relay's e.g. for NAT

#[macro_use]
extern crate failure;

use failure::Error;
use std::sync::{Arc, Mutex};

pub mod error;

pub type SerializedAddress = String;
pub type TransportAddress = String;
pub type SerializedMessage = String;

/// this closure type will get called when the send completes and the parameter will be the response message (or error)
type SendResponseClosure = Box<FnMut(Result<SerializedMessage, Error>) -> Option<Error> + Send>;

/// this closure type gets called when a new message arrives, you can respond with a Message or an error.
type ReceiveClosure =
    Box<FnMut(&SerializedAddress, &SerializedMessage) -> Result<SerializedMessage, Error> + Send>;

pub trait Node {
    fn get_address(&self) -> SerializedAddress;
    fn get_transport_address(&self) -> TransportAddress;
}

pub struct Handler {
    pub handler: Option<ReceiveClosure>,
}

impl Handler {
    fn handle(
        &mut self,
        from: &SerializedAddress,
        message: &SerializedMessage,
    ) -> Result<SerializedMessage, Error> {
        match self.handler {
            None => bail!("fish"),
            Some(ref mut handler) => (handler)(from, message),
        }
    }
}

pub trait Transport {
    //    fn initialize(config);
    fn new_node(&mut self, addr: SerializedAddress, handler: Option<Handler>) -> Result<(), Error>;
    fn send(
        &mut self,
        from: &SerializedAddress,
        to: &SerializedAddress,
        msg: SerializedMessage,
        callback: SendResponseClosure,
    ) -> Result<(), Error>;
    fn deliver(
        &mut self,
        from: &SerializedAddress,
        to: &SerializedAddress,
        message: SerializedMessage,
    ) -> Result<SerializedMessage, Error>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    //    use error::NetworkError;

    pub struct SimpleNode {
        hc_addr: SerializedAddress,
        transport_addr: u32,
    }

    impl Node for SimpleNode {
        fn get_address(&self) -> SerializedAddress {
            format!("{}", self.hc_addr)
        }

        fn get_transport_address(&self) -> TransportAddress {
            format!("{}", self.transport_addr)
        }
    }

    pub struct SimpleTransport {
        nodes: Vec<Arc<SimpleNode>>,
        handlers: HashMap<SerializedAddress, Handler>,
    }

    impl SimpleTransport {
        pub fn new() -> SimpleTransport {
            SimpleTransport {
                nodes: Vec::new(),
                handlers: HashMap::new(),
            }
        }
        pub fn exists(&self, addr: &SerializedAddress) -> bool {
            if let Some(_node) = self.nodes.iter().find(|node| node.get_address() == *addr) {
                true
            } else {
                false
            }
        }
    }

    impl Transport for SimpleTransport {
        fn send(
            &mut self,
            from: &SerializedAddress,
            to: &SerializedAddress,
            msg: SerializedMessage,
            mut callback: SendResponseClosure,
        ) -> Result<(), Error> {
            if self.exists(from) {
                let result = callback(self.deliver(from, to, msg));
                if let Some(err) = result {
                    Err(err)
                } else {
                    Ok(())
                }
            } else {
                bail!("can't send from unknown node {}", from);
            }
        }

        fn deliver(
            &mut self,
            from: &SerializedAddress,
            to: &SerializedAddress,
            message: SerializedMessage,
        ) -> Result<SerializedMessage, Error> {
            if !self.handlers.contains_key(to) {
                bail!("no handler for {}", to);
            }
            let mut_h = self.handlers.get_mut(to);
            if let Some(h) = mut_h {
                h.handle(from, &message)
            } else {
                bail!("error while getting mutable handler for {}", to);
            }
        }
        fn new_node(
            &mut self,
            serialized_addr: SerializedAddress,
            handler: Option<Handler>,
        ) -> Result<(), Error> {
            if serialized_addr == "" {
                bail!("bad holochain address")
            }
            if let Some(h) = handler {
                self.handlers.insert(serialized_addr.clone(), h);
            }
            let node = Arc::new(SimpleNode {
                transport_addr: self.nodes.len() as u32,
                hc_addr: serialized_addr,
            });
            self.nodes.push(node.clone());
            Ok(())
        }
    }

    #[test]
    fn can_create_node() {
        let mut net = SimpleTransport::new();
        let addr = "Qm..192".into();
        let result = net.new_node(addr, None);
        match result {
            Ok(()) => {
                assert_eq!(net.nodes.len(), 1);
            }
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn can_fail_on_create_node() {
        let mut net = SimpleTransport::new();
        let node = net.new_node("".into(), None);
        match node {
            Ok(()) => assert!(false),
            Err(err) => assert_eq!(err.to_string(), "bad holochain address"),
        }
    }

    #[test]
    fn can_receive_delivered_messages() {
        let mut net = SimpleTransport::new();
        let msgs = Arc::new(Mutex::new(Vec::new()));

        let msgs1 = msgs.clone();
        let callback = move |from: &SerializedAddress, message: &SerializedMessage| {
            let return_msg: SerializedMessage = format!("{} sent: {}", from, message);
            (*msgs1.lock().unwrap()).push(message.clone());
            Ok(return_msg)
        };

        let node_to = "Qm..191".to_string();
        let node_from = "Qm..192".to_string();
        net.new_node(
            node_to.clone(),
            Some(Handler {
                handler: Some(Box::new(callback)),
            }),
        ).unwrap();

        assert_eq!(net.handlers.len(), 1);

        match net.deliver(&node_from, &node_to, "foo message".into()) {
            Ok(msg) => assert_eq!("Qm..192 sent: foo message", msg),
            Err(_) => assert!(false),
        }
        assert_eq!(msgs.lock().unwrap()[0], "foo message".to_string());

        match net.deliver(&node_from, &"3333".to_string(), "foo message".into()) {
            Ok(_) => assert!(false),
            Err(err) => assert_eq!(err.to_string(), "no handler for 3333"),
        }
    }

    #[test]
    fn fails_to_send_from_uninitialized_nodes() {
        let mut net = SimpleTransport::new();
        let node_to = "Qm..191".to_string();
        let node_from = "Qm..192".to_string();
        let callback = move |_result| None;
        match net.send(
            &node_from,
            &node_to,
            "foo message".into(),
            Box::new(callback),
        ) {
            Ok(_) => assert!(false),
            Err(err) => assert_eq!(err.to_string(), "can't send from unknown node Qm..192"),
        }
    }

    #[test]
    fn can_send() {
        let mut net = SimpleTransport::new();
        let msgs = Arc::new(Mutex::new(Vec::new()));

        let msgs1 = msgs.clone();
        let callback = move |from: &SerializedAddress, message: &SerializedMessage| {
            if *message == "fail me" {
                bail!("handler failure!")
            }
            let return_msg: SerializedMessage = format!("{} sent: {}", from, message);
            (*msgs1.lock().unwrap()).push(message.clone());
            Ok(return_msg)
        };

        let node_to = "Qm..191".to_string();
        let node_from = "Qm..192".to_string();
        net.new_node(
            node_to.clone(),
            Some(Handler {
                handler: Some(Box::new(callback)),
            }),
        ).unwrap();

        net.new_node(node_from.clone(), None).unwrap();

        assert_eq!(net.handlers.len(), 1);

        let send_callback1 = move |response: Result<SerializedMessage, Error>| {
            match response {
                Err(_) => assert!(false),
                Ok(response_msg) => assert_eq!(response_msg, "Qm..192 sent: foo message"),
            };
            None
        };

        match net.send(
            &node_from,
            &node_to,
            "foo message".into(),
            Box::new(send_callback1),
        ) {
            Ok(result) => assert_eq!(result, ()),
            Err(_) => assert!(false),
        }
        assert_eq!(msgs.lock().unwrap()[0], "foo message".to_string());

        // test that a handler can send and error back to the sender
        let send_callback2 = move |response: Result<SerializedMessage, Error>| {
            match response {
                Ok(_) => assert!(false),
                Err(err) => assert_eq!(err.to_string(), "handler failure!"),
            };
            None
        };

        match net.send(
            &node_from,
            &node_to,
            "fail me".into(),
            Box::new(send_callback2),
        ) {
            Ok(result) => assert_eq!(result, ()),
            Err(_) => assert!(false),
        }
    }

}
