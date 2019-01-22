//! provides fake in-memory p2p worker for use in scenario testing

use crate::memory_server::*;
use holochain_core_types::{cas::content::Address, json::JsonString};
use holochain_net_connection::{
    json_protocol::JsonProtocol,
    net_connection::{NetHandler, NetWorker},
    protocol::Protocol,
    NetResult,
};
use std::{
    collections::{hash_map::Entry, HashMap},
    convert::TryFrom,
    sync::{mpsc, Mutex},
};

/// a p2p worker for mocking in-memory scenario tests
pub struct InMemoryWorker {
    handler: NetHandler,
    receiver_per_dna: HashMap<Address, mpsc::Receiver<Protocol>>,
    server_name: String,
}

impl NetWorker for InMemoryWorker {
    /// we got a message from holochain core
    /// forward to our in-memory server
    fn receive(&mut self, data: Protocol) -> NetResult<()> {
        let server_map = MEMORY_SERVER_MAP.read().unwrap();
        let mut server = server_map
            .get(&self.server_name)
            .expect("InMemoryServer should have been initialized by now")
            .lock()
            .unwrap();
        if let Ok(json_msg) = JsonProtocol::try_from(&data) {
            if let JsonProtocol::TrackDna(track_msg) = json_msg {
                match self
                    .receiver_per_dna
                    .entry(track_msg.dna_address.to_owned())
                {
                    Entry::Occupied(_) => (),
                    Entry::Vacant(e) => {
                        let (tx, rx) = mpsc::channel();
                        server.register(&track_msg.dna_address, &track_msg.agent_id, tx)?;
                        e.insert(rx);
                    }
                };
            }
        }
        server.serve(data)?;
        Ok(())
    }

    /// check for messages from our InMemoryServer
    fn tick(&mut self) -> NetResult<bool> {
        let mut did_something = false;
        for (_, receiver) in self.receiver_per_dna.iter_mut() {
            if let Ok(data) = receiver.try_recv() {
                did_something = true;
                (self.handler)(Ok(data))?;
            }
        }
        Ok(did_something)
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

    use holochain_core_types::cas::content::Address;
    use holochain_net_connection::json_protocol::{
        DhtData, DhtMetaData, FetchDhtData, FetchDhtMetaData, JsonProtocol, MessageData,
        SuccessResultData, TrackDnaData,
    };

    fn example_dna_address() -> Address {
        "blabladnaAddress".into()
    }

    static AGENT_ID_1: &'static str = "agent-hash-test-1";
    static AGENT_ID_2: &'static str = "agent-hash-test-2";

    #[test]
    #[cfg_attr(tarpaulin, skip)]
    fn can_memory_double_track() {
        // setup client 1
        let memory_config = &JsonString::from(P2pConfig::unique_memory_backend_string());
        let (handler_send_1, handler_recv_1) = mpsc::channel::<Protocol>();

        let mut memory_worker_1 = Box::new(
            InMemoryWorker::new(
                Box::new(move |r| {
                    handler_send_1.send(r?)?;
                    Ok(())
                }),
                memory_config,
            )
            .unwrap(),
        );

        // First Track
        memory_worker_1
            .receive(
                JsonProtocol::TrackDna(TrackDnaData {
                    dna_address: example_dna_address(),
                    agent_id: AGENT_ID_1.to_string(),
                })
                .into(),
            )
            .unwrap();

        // Should receive PeerConnected
        memory_worker_1.tick().unwrap();
        let _res = JsonProtocol::try_from(handler_recv_1.recv().unwrap()).unwrap();

        // Second Track
        memory_worker_1
            .receive(
                JsonProtocol::TrackDna(TrackDnaData {
                    dna_address: example_dna_address(),
                    agent_id: AGENT_ID_1.to_string(),
                })
                .into(),
            )
            .unwrap();

        memory_worker_1.tick().unwrap();
    }

    #[test]
    #[cfg_attr(tarpaulin, skip)]
    fn can_memory_network_flow() {
        // setup client 1
        let memory_config = &JsonString::from(P2pConfig::unique_memory_backend_string());
        let (handler_send_1, handler_recv_1) = mpsc::channel::<Protocol>();

        let mut memory_worker_1 = Box::new(
            InMemoryWorker::new(
                Box::new(move |r| {
                    handler_send_1.send(r?)?;
                    Ok(())
                }),
                memory_config,
            )
            .unwrap(),
        );

        memory_worker_1
            .receive(
                JsonProtocol::TrackDna(TrackDnaData {
                    dna_address: example_dna_address(),
                    agent_id: AGENT_ID_1.to_string(),
                })
                .into(),
            )
            .unwrap();
        // Should receive PeerConnected
        memory_worker_1.tick().unwrap();
        let _res = JsonProtocol::try_from(handler_recv_1.recv().unwrap()).unwrap();

        // setup client 2
        let (handler_send_2, handler_recv_2) = mpsc::channel::<Protocol>();
        let mut memory_worker_2 = Box::new(
            InMemoryWorker::new(
                Box::new(move |r| {
                    handler_send_2.send(r?)?;
                    Ok(())
                }),
                memory_config,
            )
            .unwrap(),
        );
        memory_worker_2
            .receive(
                JsonProtocol::TrackDna(TrackDnaData {
                    dna_address: example_dna_address(),
                    agent_id: AGENT_ID_2.to_string(),
                })
                .into(),
            )
            .unwrap();
        // Should receive PeerConnected
        memory_worker_1.tick().unwrap();
        let _res = JsonProtocol::try_from(handler_recv_1.recv().unwrap()).unwrap();
        memory_worker_2.tick().unwrap();
        let _res = JsonProtocol::try_from(handler_recv_2.recv().unwrap()).unwrap();

        // node2node:  send & receive
        memory_worker_1
            .receive(
                JsonProtocol::SendMessage(MessageData {
                    dna_address: example_dna_address(),
                    to_agent_id: AGENT_ID_2.to_string(),
                    from_agent_id: AGENT_ID_1.to_string(),
                    msg_id: "yada".to_string(),
                    data: json!("hello"),
                })
                .into(),
            )
            .unwrap();

        memory_worker_2.tick().unwrap();

        let res = JsonProtocol::try_from(handler_recv_2.recv().unwrap()).unwrap();

        if let JsonProtocol::HandleSendMessage(msg) = res {
            memory_worker_2
                .receive(
                    JsonProtocol::HandleSendMessageResult(MessageData {
                        dna_address: msg.dna_address,
                        to_agent_id: msg.from_agent_id,
                        from_agent_id: AGENT_ID_2.to_string(),
                        msg_id: msg.msg_id,
                        data: json!(format!("echo: {}", msg.data.to_string())),
                    })
                    .into(),
                )
                .unwrap();
        } else {
            println!("Did not expect to receive: {:?}", res);
            panic!("bad msg");
        }

        memory_worker_1.tick().unwrap();

        let res = JsonProtocol::try_from(handler_recv_1.recv().unwrap()).unwrap();

        if let JsonProtocol::SendMessageResult(msg) = res {
            assert_eq!("\"echo: \\\"hello\\\"\"".to_string(), msg.data.to_string());
        } else {
            println!("Did not expect to receive: {:?}", res);
            panic!("bad msg");
        }

        // -- dht get -- //

        memory_worker_2
            .receive(
                JsonProtocol::FetchDhtData(FetchDhtData {
                    request_id: "yada".to_string(),
                    dna_address: example_dna_address(),
                    requester_agent_id: AGENT_ID_2.to_string(),
                    data_address: "hello".to_string(),
                })
                .into(),
            )
            .unwrap();

        memory_worker_1.tick().unwrap();

        let res = JsonProtocol::try_from(handler_recv_1.recv().unwrap()).unwrap();

        if let JsonProtocol::HandleFetchDhtData(msg) = res {
            memory_worker_1
                .receive(
                    JsonProtocol::HandleFetchDhtDataResult(DhtData {
                        request_id: msg.request_id.clone(),
                        dna_address: msg.dna_address.clone(),
                        provider_agent_id: msg.requester_agent_id.clone(),
                        data_address: msg.data_address.clone(),
                        data_content: json!(format!("data-for: {}", msg.address)),
                    })
                    .into(),
                )
                .unwrap();
        } else {
            println!("Did not expect to receive: {:?}", res);
            panic!("bad msg");
        }

        memory_worker_2.tick().unwrap();

        let res = JsonProtocol::try_from(handler_recv_2.recv().unwrap()).unwrap();

        if let JsonProtocol::FetchDhtDataResult(msg) = res {
            assert_eq!("\"data-for: hello\"".to_string(), msg.content.to_string());
        } else {
            println!("Did not expect to receive: {:?}", res);
            panic!("bad msg");
        }

        // -- dht publish / store -- //

        memory_worker_2
            .receive(
                JsonProtocol::PublishDhtData(DhtData {
                    request_id: "yada".to_string(),
                    dna_address: example_dna_address(),
                    provider_agent_id: AGENT_ID_2.to_string(),
                    data_address: "hello".to_string(),
                    data_content: json!("test-data"),
                })
                .into(),
            )
            .unwrap();

        memory_worker_1.tick().unwrap();
        memory_worker_2.tick().unwrap();

        let res1 = JsonProtocol::try_from(handler_recv_1.recv().unwrap()).unwrap();
        let res2 = JsonProtocol::try_from(handler_recv_2.recv().unwrap()).unwrap();

        assert_eq!(res1, res2);

        if let JsonProtocol::HandleStoreDhtData(msg) = res1 {
            memory_worker_1
                .receive(
                    JsonProtocol::SuccessResult(SuccessResultData {
                        msg_id: msg.request_id.clone(),
                        dna_address: msg.dna_address.clone(),
                        to_agent_id: msg.provider_agent_id.clone(),
                        success_info: json!("signature here"),
                    })
                    .into(),
                )
                .unwrap();
        } else {
            println!("Did not expect to receive: {:?}", res1);
            panic!("bad msg");
        }

        memory_worker_2.tick().unwrap();
        let res = JsonProtocol::try_from(handler_recv_2.recv().unwrap()).unwrap();

        if let JsonProtocol::SuccessResult(msg) = res {
            assert_eq!("\"signature here\"", &msg.success_info.to_string())
        } else {
            println!("Did not expect to receive: {:?}", res);
            panic!("bad msg");
        }

        // -- dht meta get -- //

        memory_worker_2
            .receive(
                JsonProtocol::FetchDhtMeta(FetchDhtMetaData {
                    request_id: "yada".to_string(),
                    dna_address: example_dna_address(),
                    requester_agent_id: AGENT_ID_2.to_string(),
                    data_address: "hello".to_string(),
                    attribute: "link:test".to_string(),
                })
                .into(),
            )
            .unwrap();

        memory_worker_1.tick().unwrap();

        let res = JsonProtocol::try_from(handler_recv_1.recv().unwrap()).unwrap();

        if let JsonProtocol::HandleFetchDhtMeta(msg) = res {
            memory_worker_1
                .receive(
                    JsonProtocol::HandleFetchDhtMetaResult(DhtMetaData {
                        request_id: msg.request_id.clone(),
                        dna_address: msg.dna_address.clone(),
                        requester_agent_id: msg.requester_agent_id.clone(),
                        provider_agent_id: AGENT_ID_1.to_string(),
                        data_address: msg.data_address.clone(),
                        attribute: msg.attribute.clone(),
                        content: json!(format!("meta-data-for: {}", msg.address)),
                    })
                    .into(),
                )
                .unwrap();
        } else {
            println!("Did not expect to receive: {:?}", res);
            panic!("bad msg");
        }

        memory_worker_2.tick().unwrap();

        let res = JsonProtocol::try_from(handler_recv_2.recv().unwrap()).unwrap();

        if let JsonProtocol::FetchDhtMetaResult(msg) = res {
            assert_eq!(
                "\"meta-data-for: hello\"".to_string(),
                msg.content.to_string()
            );
        } else {
            println!("Did not expect to receive: {:?}", res);
            panic!("bad msg");
        }

        // -- dht meta publish / store -- //

        memory_worker_2
            .receive(
                JsonProtocol::PublishDhtMeta(DhtMetaData {
                    request_id: "yada".to_string(),
                    dna_address: example_dna_address(),
                    requester_agent_id: AGENT_ID_2.to_string(),
                    provider_agent_id: AGENT_ID_1.to_string(),
                    data_address: "hello".to_string(),
                    attribute: "link:test".to_string(),
                    content: json!("test-data"),
                })
                .into(),
            )
            .unwrap();

        memory_worker_1.tick().unwrap();
        memory_worker_2.tick().unwrap();

        let res1 = JsonProtocol::try_from(handler_recv_1.recv().unwrap()).unwrap();
        let res2 = JsonProtocol::try_from(handler_recv_2.recv().unwrap()).unwrap();

        assert_eq!(res1, res2);

        if let JsonProtocol::HandleStoreDhtMeta(msg) = res1 {
            memory_worker_1
                .receive(
                    JsonProtocol::SuccessResult(SuccessResultData {
                        msg_id: msg.request_id.clone(),
                        dna_address: msg.dna_address.clone(),
                        to_agent_id: msg.requester_agent_id.clone(),
                        success_info: json!("signature here"),
                    })
                    .into(),
                )
                .unwrap();
        } else {
            println!("Did not expect to receive: {:?}", res1);
            panic!("bad msg");
        }

        memory_worker_2.tick().unwrap();
        let res = JsonProtocol::try_from(handler_recv_2.recv().unwrap()).unwrap();

        if let JsonProtocol::SuccessResult(msg) = res {
            assert_eq!("\"signature here\"", &msg.success_info.to_string())
        } else {
            println!("Did not expect to receive: {:?}", res);
            panic!("bad msg");
        }

        // cleanup
        memory_worker_1.stop().unwrap();
        memory_worker_2.stop().unwrap();
    }
}
