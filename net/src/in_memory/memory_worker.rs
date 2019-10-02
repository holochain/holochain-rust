//! provides fake in-memory p2p worker for use in scenario testing

use super::memory_server::*;
use crate::connection::{
    net_connection::{NetHandler, NetWorker},
    NetResult,
};

use lib3h_protocol::{protocol_client::Lib3hClientProtocol, protocol_server::Lib3hServerProtocol};

use holochain_json_api::json::JsonString;
use holochain_persistence_api::{cas::content::Address, hash::HashString};
use std::{
    collections::{hash_map::Entry, HashMap},
    sync::{mpsc, Mutex},
};

/// a p2p worker for mocking in-memory scenario tests
#[allow(non_snake_case)]
pub struct InMemoryWorker {
    handler: NetHandler,
    receiver_per_dna: HashMap<Address, mpsc::Receiver<Lib3hServerProtocol>>,
    server_name: String,
    can_send_P2pReady: bool,
}

impl NetWorker for InMemoryWorker {
    /// we got a message from holochain core
    /// forward to our in-memory server
    fn receive(&mut self, data: Lib3hClientProtocol) -> NetResult<()> {
        // InMemoryWorker doesn't have to do anything on shutdown
        if data == Lib3hClientProtocol::Shutdown {
            self.handler.handle(Ok(Lib3hServerProtocol::Terminated))?;
            return Ok(());
        }
        let server_map = MEMORY_SERVER_MAP.read().unwrap();
        let mut server = server_map
            .get(&self.server_name)
            .expect("InMemoryServer should have been initialized by now")
            .lock()
            .unwrap();
        match &data {
            Lib3hClientProtocol::JoinSpace(track_msg) => {
                let dna_address: HashString = track_msg.space_address.clone();
                match self.receiver_per_dna.entry(dna_address.clone()) {
                    Entry::Occupied(_) => (),
                    Entry::Vacant(e) => {
                        let (tx, rx) = mpsc::channel();
                        println!("register_chain: {}::{}", dna_address, track_msg.agent_id);
                        server.register_chain(&dna_address, &track_msg.agent_id, tx)?;
                        e.insert(rx);
                    }
                };
            }
            _ => (),
        };
        // Serve
        server.serve(data.clone())?;
        // After serve
        match &data {
            Lib3hClientProtocol::LeaveSpace(untrack_msg) => {
                let dna_address: HashString = untrack_msg.space_address.clone();
                match self.receiver_per_dna.entry(dna_address.clone()) {
                    Entry::Vacant(_) => (),
                    Entry::Occupied(e) => {
                        server.unregister_chain(&dna_address, &untrack_msg.agent_id);
                        e.remove();
                    }
                };
            }
            _ => (),
        };
        // Done
        Ok(())
    }

    /// check for messages from our InMemoryServer
    fn tick(&mut self) -> NetResult<bool> {
        // Send p2pready on first tick
        if self.can_send_P2pReady {
            self.can_send_P2pReady = false;
            self.handler.handle(Ok(Lib3hServerProtocol::P2pReady))?;
        }
        // check for messages from our InMemoryServer
        let mut did_something = false;
        for (_, receiver) in self.receiver_per_dna.iter_mut() {
            if let Ok(data) = receiver.try_recv() {
                did_something = true;
                self.handler.handle(Ok(data))?;
            }
        }
        Ok(did_something)
    }
    
    /// Set the advertise as worker's endpoint
    fn p2p_endpoint(&self) -> Option<url::Url> {
        None
    }

    /// stop the net worker
    fn stop(self: Box<Self>) -> NetResult<()> {
        Ok(())
    }

    /// Set server's name as worker's endpoint
    fn endpoint(&self) -> Option<String> {
        Some(self.server_name.clone())
    }
}

impl InMemoryWorker {
    /// create a new memory worker connected to an in-memory server
    pub fn new(handler: NetHandler, backend_config: &JsonString) -> NetResult<Self> {
        // Get server name from config
        let config: serde_json::Value = serde_json::from_str(backend_config.into())?;
        // println!("InMemoryWorker::new() config = {:?}", config);
        let server_name = config["serverName"]
            .as_str()
            .unwrap_or("(unnamed)")
            .to_string();
        // Create server with that name if it doesn't already exist
        let mut server_map = MEMORY_SERVER_MAP.write().unwrap();
        if !server_map.contains_key(&server_name) {
            server_map.insert(
                server_name.clone(),
                Mutex::new(InMemoryServer::new(server_name.clone())),
            );
        }
        let mut server = server_map
            .get(&server_name)
            .expect("InMemoryServer should exist")
            .lock()
            .unwrap();
        server.clock_in();

        Ok(InMemoryWorker {
            handler,
            receiver_per_dna: HashMap::new(),
            server_name,
            can_send_P2pReady: true,
        })
    }
}

// unregister on Drop
impl Drop for InMemoryWorker {
    fn drop(&mut self) {
        let server_map = MEMORY_SERVER_MAP.read().unwrap();
        let mut server = server_map
            .get(&self.server_name)
            .expect("InMemoryServer should exist")
            .lock()
            .unwrap();
        server.clock_out();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::p2p_config::P2pConfig;
    use crossbeam_channel::unbounded;
    use holochain_persistence_api::{cas::content::Address, hash::HashString};
    use lib3h_protocol::data_types::SpaceData;

    fn example_dna_address() -> Address {
        "QmYsFu7QGaVeUUac1E4BWST7BR38cYvzRaaTc3YS9WqsTu".into()
    }

    static AGENT_ID_1: &'static str = "QmY6MfiuhHnQ1kg7RwNZJNUQhwDxTFL45AAPnpJMNPEoxk";
    // TODO - AgentIds need to be HcSyada base32 format
    //        currently HashString try_into Vec<u8> is doing only base58
    //static AGENT_ID_1: &'static str = "HcScIkRaAaaaaaaaaaAaaaAAAAaaaaaaaaAaaaaAaaaaaaaaAaaAAAAatzu4aqa";

    #[test]
    #[cfg_attr(tarpaulin, skip)]
    fn can_memory_worker_double_track() {
        // setup client 1
        let memory_config = &JsonString::from(P2pConfig::unique_memory_backend_json());
        let (handler_send_1, handler_recv_1) = unbounded::<Lib3hServerProtocol>();

        let mut memory_worker_1 = Box::new(
            InMemoryWorker::new(
                NetHandler::new(Box::new(move |r| {
                    handler_send_1.send(r?)?;
                    Ok(())
                })),
                memory_config,
            )
            .unwrap(),
        );

        // Should receive p2pready on first tick
        memory_worker_1.tick().unwrap();
        let message = handler_recv_1.recv().unwrap();
        assert!(match message {
            Lib3hServerProtocol::P2pReady => true,
            _ => false,
        });
        // First Track
        memory_worker_1
            .receive(Lib3hClientProtocol::JoinSpace(SpaceData {
                request_id: "test_req1".to_string(),
                space_address: example_dna_address(),
                agent_id: HashString::from(AGENT_ID_1),
            }))
            .unwrap();

        // Should receive PeerConnected
        memory_worker_1.tick().unwrap();
        let _res: Lib3hServerProtocol = handler_recv_1.recv().unwrap();

        // Second Track
        memory_worker_1
            .receive(Lib3hClientProtocol::JoinSpace(SpaceData {
                request_id: "test_req2".to_string(),
                space_address: example_dna_address(),
                agent_id: HashString::from(AGENT_ID_1),
            }))
            .unwrap();

        memory_worker_1.tick().unwrap();
    }
}
