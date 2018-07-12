//! holochain_net is a library that defines an abstract networking layer for
//! different network transports and implements a PeerStore for mapping and
//! managing the topology of transport layers with regard to relay's e.g. for NAT

#[macro_use]
extern crate failure;

use failure::Error;
use std::sync::{Mutex, Arc};

pub mod error;

pub type SerializedAddress = String;
pub type SerializedMessage = String;

/*
pub struct Message {
    // envelope (meta data plus signature(s))
    // contents (date, type, body)
    contents: String
}


/// this closure type will get called when the send completes and the parameter will be the response message (or error)
type SendResponseClosure = Box<FnMut(Result<Message,NetworkError>) -> Option<NetworkError> + Send>;

 */

/// this closure type gets called when a new message arrives, you can respond with a Message or an error.
type ReceiveClosure = Box<FnMut(&SerializedAddress, &SerializedMessage) -> Result<SerializedMessage,Error> + Send>;

pub trait Node {
//    fn send(&self, to: &Node, msg: Message, callback: SendResponseClosure);
    fn deliver(&mut self, from: SerializedAddress, message:SerializedMessage) -> Result<SerializedMessage,Error>;
    fn receive(&mut self, handler: ReceiveClosure);
    fn get_address(&self) ->SerializedAddress;
}

pub struct Handler {
    pub handler: Option<ReceiveClosure>,
}

impl Handler {
    fn handle(&mut self,from:&SerializedAddress,message:&SerializedMessage) -> Result<SerializedMessage,Error> {
        match self.handler {
            None => bail!("fish"),
            Some(ref mut handler) => (handler)(from,message)
        }
    }
}

pub trait Transport {
//    fn initialize(config);
    fn new_node(&mut self, addr: SerializedAddress,handler:Option<Handler>) -> Result< Arc<Box<Node>>, Error>;
 //   fn receive(&mut self,node: Arc<Node>, handler: ReceiveClosure);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use error::NetworkError;



//    #[derive(Copy,Clone,Debug)]
    pub struct SimpleNode {
        addr: u32,
        handler: Handler,
    }

    impl Node for SimpleNode {
/*        fn send(&self, to: &Node, msg: Message, callback: SendResponseClosure) {

        }
         */

        fn deliver(&mut self, from: SerializedAddress, message:SerializedMessage)  -> Result<SerializedMessage,Error> {
            self.handler.handle(&from,&message)
        }

        fn receive(&mut self, handler: ReceiveClosure) {
            self.handler = Handler{handler:Some(handler)};
        }

        fn get_address(&self) -> SerializedAddress {
            format!("{}",self.addr)
        }
    }

    pub struct SimpleTransport {
        nodes: Vec<Arc<Box<SimpleNode>>>,
        handlers: Vec<(SerializedAddress,Handler)>
    }

    impl SimpleTransport {
        pub fn new() ->  SimpleTransport {
            SimpleTransport{nodes: Vec::new(),handlers: Vec::new()}
        }

        pub fn node_deliver(&mut self,mut to: SimpleNode,from:SimpleNode,message:SerializedMessage) -> Result<SerializedMessage,Error> {
//            let to_addr = to.get_address();
            to.handler.handle(&from.get_address(),&message)


//            Ok("bogus message".into())
            //            let closure = self.handlers.get_mut(&to.get_address()).unwrap();
            /*

            let x= Arc::get_mut(&mut to);
            match x {
            Some(y) => {
            match (*y).handler {
            Some(zz) => {
            let z = Arc::get_mut(&mut zz);
            match z {
            Some(q) => {
            q(&from,&message)
        },
            None => panic!("uz")
        }
        },
            None => panic!("foz")

        }
        },
            None => panic!("unable to mutate node"),
        }
            //            x.handler.unwrap()(&from,&message)
             */
        }

        pub fn deliver(&mut self,mut to: Arc<SimpleNode>,from:SimpleNode,message:SerializedMessage) -> Result<SerializedMessage,Error> {

            let mut return_val : Result<SerializedMessage,Error> = Err(NetworkError::GenericError{error :"no handler for address".to_string()}.into());
//            let closure = self.handlers.get_mut(&(*to).get_address()).unwrap();
         //   let result = closure(&from,&message);
            Ok("fish".into())
/*
            let to_addr = (*to).get_address();
            self.handlers = self.handlers.into_iter().map(|(address, mut handler)| {
                if address == to_addr {
                    return_val = handler.handle(&from,&message);
                }
                (address,handler)
            }).collect::<Vec<(SerializedAddress,Handler)>>();
            return_val*/
        }
    }

    impl Transport for SimpleTransport {
    /*    fn receive(&mut self,node: Arc<Node>, handler: ReceiveClosure) {
            self.handlers.push(((*node).get_address(),Handler{handler:Some(handler)}));
        }*/

       fn new_node(&mut self, serialized_addr: SerializedAddress,handler: Option<Handler>) -> Result<Arc<Box<Node>>,Error> {
            //            let addr = self.new_address(serialized_addr);
            match serialized_addr.parse::<u32>() {
                Err(err) => bail!("bad address: {}",err.to_string()),
                Ok(addr) => {
                    let h = match handler {
                        None => Handler{handler:None},
                        Some(hh) => hh
                    };
                    let node = Arc::new(Box::new(SimpleNode{addr: addr,handler: h}));
                    self.nodes.push(node.clone());
                    Ok(node)
                }
            }
        }
    }

    #[test]
    fn can_create_node() {
        let mut net = SimpleTransport::new();
        let node = net.new_node("192".into(),None);
        match node {
            Ok(n) => {
                assert_eq!(n.get_address(),"192");
                assert_eq!(net.nodes.len(),1);
            },
            Err(_) => assert!(false)
        }
    }

    #[test]
    fn can_fail_on_create_node() {
        let mut net = SimpleTransport::new();
        let node = net.new_node("a bad address".into(),None);
        match node {
            Ok(_) =>  assert!(false),
            Err(err) =>assert_eq!(err.to_string(),"bad address: invalid digit found in string"),
        }
    }

    #[test]
    fn can_receive_via_node() {
        let mut net = SimpleTransport::new();

        let msgs = Arc::new(Mutex::new(Vec::new()));

        let msgs1 = msgs.clone();
        let callback = move |from:&SerializedAddress ,message:&SerializedMessage|{
            let return_msg : SerializedMessage = format!("{} sent: {}",from,message);
            (*msgs1.lock().unwrap()).push(return_msg.clone());
            Ok(return_msg)
        };


        let mut node = net.new_node("192".into(),Some(Handler{handler:Some(Box::new(callback))})).unwrap();

        let sending_node = net.new_node("191".into(),None).unwrap();

        //node.deliver(sending_node,"foo message".into());

        let x= Arc::get_mut(&mut node).unwrap();
        x.deliver(sending_node.get_address(),"foo message".into());

        //let x = node.get_mut();
/*        match x {
            Some(y) => y.deliver(sending_node,"foo message".into()),
            None => panic!("unable to mutate2"),
        };
*/
        assert_eq!(msgs.lock().unwrap()[0],"192 sent foo message".to_string());
    }

    #[test]
    fn can_receive_via_transport() {
        let mut net = SimpleTransport::new();
        let msgs = Arc::new(Mutex::new(Vec::new()));

        let msgs1 = msgs.clone();
        let callback = move |from:&SerializedAddress ,message:&SerializedMessage|{
            let return_msg : SerializedMessage = format!("{} sent: {}",from,message);
            (*msgs1.lock().unwrap()).push(return_msg.clone());
            Ok(return_msg)
        };

        let node = net.new_node("192".into(),Some(Handler{handler:Some(Box::new(callback))})).unwrap();
        //  let sending_node = net.new_node("191".into()).unwrap();

//        net.receive(node,Box::new(callback));
        node.deliver(node.get_address(),"foo message".into());

        assert_eq!(msgs.lock().unwrap()[0],"192 sent foo message".to_string());
    }

}
