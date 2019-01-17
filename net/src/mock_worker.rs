//! provides fake in-memory p2p worker for use in scenario testing

use crate::mock_system::*;
use holochain_core_types::json::JsonString;
use holochain_net_connection::{
    json_protocol::JsonProtocol,
    net_connection::{NetHandler, NetWorker},
    protocol::Protocol,
    NetResult,
};
use std::{
    convert::TryFrom,
    sync::{mpsc, Mutex},
};

/// a p2p worker for mocking in-memory scenario tests
pub struct MockWorker {
    handler: NetHandler,
    mock_msgs: Vec<mpsc::Receiver<Protocol>>,
    network_name: String,
}

impl NetWorker for MockWorker {
    /// we got a message from holochain core
    /// forward to our mock singleton
    fn receive(&mut self, data: Protocol) -> NetResult<()> {
        let map_lock = MOCK_MAP.read().unwrap();
        let mut mock = map_lock
            .get(&self.network_name)
            .expect("MockSystem should have been initialized by now")
            .lock()
            .unwrap();
        if let Ok(json_msg) = JsonProtocol::try_from(&data) {
            if let JsonProtocol::TrackDna(track_msg) = json_msg {
                let (tx, rx) = mpsc::channel();
                self.mock_msgs.push(rx);
                mock.register(&track_msg.dna_address, &track_msg.agent_id, tx)?;
            }
        }
        mock.handle(data)?;
        Ok(())
    }

    /// check for messages from our mock singleton
    fn tick(&mut self) -> NetResult<bool> {
        let mut did_something = false;
        for msg in self.mock_msgs.iter_mut() {
            if let Ok(data) = msg.try_recv() {
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

    /// Set network's name as worker's endpoint
    fn endpoint(&self) -> Option<String> {
        Some(self.network_name.clone())
    }
}

impl MockWorker {
    /// create a new mock worker... no configuration required
    pub fn new(handler: NetHandler, network_config: &JsonString) -> NetResult<Self> {
        let config: serde_json::Value = serde_json::from_str(network_config.into())?;
        let network_name = config["networkName"]
            .as_str()
            .unwrap_or("(unnamed)")
            .to_string();

        let mut map_lock = MOCK_MAP.write().unwrap();
        if !map_lock.contains_key(&network_name) {
            map_lock.insert(network_name.clone(), Mutex::new(MockSystem::new()));
        }

        Ok(MockWorker {
            handler,
            mock_msgs: Vec::new(),
            network_name,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::p2p_config::P2pConfig;

    use holochain_core_types::cas::content::Address;
    use holochain_net_connection::json_protocol::{
        DhtData, DhtMetaData, GetDhtData, GetDhtMetaData, JsonProtocol, MessageData,
        SuccessResultData, TrackDnaData,
    };

    fn example_dna_address() -> Address {
        "blabladnaAddress".into()
    }

    static AGENT_ID_1: &'static str = "agent-hash-test-1";
    static AGENT_ID_2: &'static str = "agent-hash-test-2";

    #[test]
    #[cfg_attr(tarpaulin, skip)]
    fn it_mock_networker_flow() {
        // setup client 1
        let config = &JsonString::from(P2pConfig::unique_mock_as_string());
        let (handler_send_1, handler_recv_1) = mpsc::channel::<Protocol>();

        let mut mock_worker_1 = Box::new(
            MockWorker::new(
                Box::new(move |r| {
                    handler_send_1.send(r?)?;
                    Ok(())
                }),
                config,
            )
            .unwrap(),
        );

        mock_worker_1
            .receive(
                JsonProtocol::TrackDna(TrackDnaData {
                    dna_address: example_dna_address(),
                    agent_id: AGENT_ID_1.to_string(),
                })
                .into(),
            )
            .unwrap();
        // Should receive PeerConnected
        mock_worker_1.tick().unwrap();
        let _res = JsonProtocol::try_from(handler_recv_1.recv().unwrap()).unwrap();

        // setup client 2
        let (handler_send_2, handler_recv_2) = mpsc::channel::<Protocol>();
        let mut mock_worker_2 = Box::new(
            MockWorker::new(
                Box::new(move |r| {
                    handler_send_2.send(r?)?;
                    Ok(())
                }),
                config,
            )
            .unwrap(),
        );
        mock_worker_2
            .receive(
                JsonProtocol::TrackDna(TrackDnaData {
                    dna_address: example_dna_address(),
                    agent_id: AGENT_ID_2.to_string(),
                })
                .into(),
            )
            .unwrap();
        // Should receive PeerConnected
        mock_worker_1.tick().unwrap();
        let _res = JsonProtocol::try_from(handler_recv_1.recv().unwrap()).unwrap();
        mock_worker_2.tick().unwrap();
        let _res = JsonProtocol::try_from(handler_recv_2.recv().unwrap()).unwrap();

        // node2node:  send & receive
        mock_worker_1
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

        mock_worker_2.tick().unwrap();

        let res = JsonProtocol::try_from(handler_recv_2.recv().unwrap()).unwrap();

        if let JsonProtocol::HandleSendMessage(msg) = res {
            mock_worker_2
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

        mock_worker_1.tick().unwrap();

        let res = JsonProtocol::try_from(handler_recv_1.recv().unwrap()).unwrap();

        if let JsonProtocol::SendMessageResult(msg) = res {
            assert_eq!("\"echo: \\\"hello\\\"\"".to_string(), msg.data.to_string());
        } else {
            println!("Did not expect to receive: {:?}", res);
            panic!("bad msg");
        }

        // -- dht get -- //

        mock_worker_2
            .receive(
                JsonProtocol::GetDhtData(GetDhtData {
                    msg_id: "yada".to_string(),
                    dna_address: example_dna_address(),
                    from_agent_id: AGENT_ID_2.to_string(),
                    address: "hello".to_string(),
                })
                .into(),
            )
            .unwrap();

        mock_worker_1.tick().unwrap();

        let res = JsonProtocol::try_from(handler_recv_1.recv().unwrap()).unwrap();

        if let JsonProtocol::HandleGetDhtData(msg) = res {
            mock_worker_1
                .receive(
                    JsonProtocol::GetDhtDataResult(DhtData {
                        msg_id: msg.msg_id.clone(),
                        dna_address: msg.dna_address.clone(),
                        agent_id: msg.from_agent_id.clone(),
                        address: msg.address.clone(),
                        content: json!(format!("data-for: {}", msg.address)),
                    })
                    .into(),
                )
                .unwrap();
        } else {
            println!("Did not expect to receive: {:?}", res);
            panic!("bad msg");
        }

        mock_worker_2.tick().unwrap();

        let res = JsonProtocol::try_from(handler_recv_2.recv().unwrap()).unwrap();

        if let JsonProtocol::GetDhtDataResult(msg) = res {
            assert_eq!("\"data-for: hello\"".to_string(), msg.content.to_string());
        } else {
            println!("Did not expect to receive: {:?}", res);
            panic!("bad msg");
        }

        // -- dht publish / store -- //

        mock_worker_2
            .receive(
                JsonProtocol::PublishDhtData(DhtData {
                    msg_id: "yada".to_string(),
                    dna_address: example_dna_address(),
                    agent_id: AGENT_ID_2.to_string(),
                    address: "hello".to_string(),
                    content: json!("test-data"),
                })
                .into(),
            )
            .unwrap();

        mock_worker_1.tick().unwrap();
        mock_worker_2.tick().unwrap();

        let res1 = JsonProtocol::try_from(handler_recv_1.recv().unwrap()).unwrap();
        let res2 = JsonProtocol::try_from(handler_recv_2.recv().unwrap()).unwrap();

        assert_eq!(res1, res2);

        if let JsonProtocol::HandleStoreDhtData(msg) = res1 {
            mock_worker_1
                .receive(
                    JsonProtocol::SuccessResult(SuccessResultData {
                        msg_id: msg.msg_id.clone(),
                        dna_address: msg.dna_address.clone(),
                        to_agent_id: msg.agent_id.clone(),
                        success_info: json!("signature here"),
                    })
                    .into(),
                )
                .unwrap();
        } else {
            println!("Did not expect to receive: {:?}", res1);
            panic!("bad msg");
        }

        mock_worker_2.tick().unwrap();
        let res = JsonProtocol::try_from(handler_recv_2.recv().unwrap()).unwrap();

        if let JsonProtocol::SuccessResult(msg) = res {
            assert_eq!("\"signature here\"", &msg.success_info.to_string())
        } else {
            println!("Did not expect to receive: {:?}", res);
            panic!("bad msg");
        }

        // -- dht meta get -- //

        mock_worker_2
            .receive(
                JsonProtocol::GetDhtMeta(GetDhtMetaData {
                    msg_id: "yada".to_string(),
                    dna_address: example_dna_address(),
                    from_agent_id: AGENT_ID_2.to_string(),
                    address: "hello".to_string(),
                    attribute: "link:test".to_string(),
                })
                .into(),
            )
            .unwrap();

        mock_worker_1.tick().unwrap();

        let res = JsonProtocol::try_from(handler_recv_1.recv().unwrap()).unwrap();

        if let JsonProtocol::HandleGetDhtMeta(msg) = res {
            mock_worker_1
                .receive(
                    JsonProtocol::GetDhtMetaResult(DhtMetaData {
                        msg_id: msg.msg_id.clone(),
                        dna_address: msg.dna_address.clone(),
                        agent_id: msg.from_agent_id.clone(),
                        from_agent_id: AGENT_ID_1.to_string(),
                        address: msg.address.clone(),
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

        mock_worker_2.tick().unwrap();

        let res = JsonProtocol::try_from(handler_recv_2.recv().unwrap()).unwrap();

        if let JsonProtocol::GetDhtMetaResult(msg) = res {
            assert_eq!(
                "\"meta-data-for: hello\"".to_string(),
                msg.content.to_string()
            );
        } else {
            println!("Did not expect to receive: {:?}", res);
            panic!("bad msg");
        }

        // -- dht meta publish / store -- //

        mock_worker_2
            .receive(
                JsonProtocol::PublishDhtMeta(DhtMetaData {
                    msg_id: "yada".to_string(),
                    dna_address: example_dna_address(),
                    agent_id: AGENT_ID_2.to_string(),
                    from_agent_id: AGENT_ID_1.to_string(),
                    address: "hello".to_string(),
                    attribute: "link:test".to_string(),
                    content: json!("test-data"),
                })
                .into(),
            )
            .unwrap();

        mock_worker_1.tick().unwrap();
        mock_worker_2.tick().unwrap();

        let res1 = JsonProtocol::try_from(handler_recv_1.recv().unwrap()).unwrap();
        let res2 = JsonProtocol::try_from(handler_recv_2.recv().unwrap()).unwrap();

        assert_eq!(res1, res2);

        if let JsonProtocol::HandleStoreDhtMeta(msg) = res1 {
            mock_worker_1
                .receive(
                    JsonProtocol::SuccessResult(SuccessResultData {
                        msg_id: msg.msg_id.clone(),
                        dna_address: msg.dna_address.clone(),
                        to_agent_id: msg.agent_id.clone(),
                        success_info: json!("signature here"),
                    })
                    .into(),
                )
                .unwrap();
        } else {
            println!("Did not expect to receive: {:?}", res1);
            panic!("bad msg");
        }

        mock_worker_2.tick().unwrap();
        let res = JsonProtocol::try_from(handler_recv_2.recv().unwrap()).unwrap();

        if let JsonProtocol::SuccessResult(msg) = res {
            assert_eq!("\"signature here\"", &msg.success_info.to_string())
        } else {
            println!("Did not expect to receive: {:?}", res);
            panic!("bad msg");
        }

        // cleanup
        mock_worker_1.stop().unwrap();
        mock_worker_2.stop().unwrap();
    }
}
