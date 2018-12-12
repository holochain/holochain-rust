//! provides fake in-memory p2p worker for use in scenario testing

use holochain_net_connection::{
    net_connection::{NetHandler, NetWorker},
    protocol::Protocol,
    protocol_wrapper::{
        DhtData, DhtMetaData, FailureResultData, GetDhtData, GetDhtMetaData, MessageData,
        P2pProtocol,
    },
    NetResult,
};

use std::{
    collections::{hash_map::Entry, HashMap},
    convert::TryFrom,
    sync::{mpsc, Mutex, MutexGuard},
};

/// hash connections by dna::agent_id
fn cat_dna_agent(dna_hash: &str, agent_id: &str) -> String {
    format!("{}::{}", dna_hash, agent_id)
}

/// a lazy_static! singleton for routing messages in-memory
struct MockSingleton {
    // keep track of senders by `dna_hash::agent_id`
    senders: HashMap<String, mpsc::Sender<Protocol>>,
    // keep track of senders as arrays by dna_hash
    senders_by_dna: HashMap<String, Vec<mpsc::Sender<Protocol>>>,
}

impl MockSingleton {
    /// create a new mock singleton
    pub fn new() -> Self {
        Self {
            senders: HashMap::new(),
            senders_by_dna: HashMap::new(),
        }
    }

    /// register a data handler with the singleton (for message routing)
    pub fn register(
        &mut self,
        dna_hash: &str,
        agent_id: &str,
        sender: mpsc::Sender<Protocol>,
    ) -> NetResult<()> {
        self.senders
            .insert(cat_dna_agent(dna_hash, agent_id), sender.clone());
        match self.senders_by_dna.entry(dna_hash.to_string()) {
            Entry::Occupied(mut e) => {
                e.get_mut().push(sender.clone());
            }
            Entry::Vacant(e) => {
                e.insert(vec![sender.clone()]);
            }
        };
        Ok(())
    }

    /// process a message
    pub fn handle(&mut self, data: Protocol) -> NetResult<()> {
        if let Ok(wrap) = P2pProtocol::try_from(&data) {
            match wrap {
                P2pProtocol::SendMessage(msg) => {
                    self.priv_handle_send(&msg)?;
                }
                P2pProtocol::HandleSendResult(msg) => {
                    self.priv_handle_send_result(&msg)?;
                }
                P2pProtocol::SuccessResult(msg) => {
                    self.priv_send_one(
                        &msg.dna_hash,
                        &msg.to_agent_id,
                        P2pProtocol::SuccessResult(msg.clone()).into(),
                    )?;
                }
                P2pProtocol::FailureResult(msg) => {
                    self.priv_send_one(
                        &msg.dna_hash,
                        &msg.to_agent_id,
                        P2pProtocol::FailureResult(msg.clone()).into(),
                    )?;
                }
                P2pProtocol::GetDht(msg) => {
                    self.priv_handle_get_dht(&msg)?;
                }
                P2pProtocol::GetDhtResult(msg) => {
                    self.priv_handle_get_dht_result(&msg)?;
                }
                P2pProtocol::PublishDht(msg) => {
                    self.priv_handle_publish_dht(&msg)?;
                }
                P2pProtocol::GetDhtMeta(msg) => {
                    self.priv_handle_get_dht_meta(&msg)?;
                }
                P2pProtocol::GetDhtMetaResult(msg) => {
                    self.priv_handle_get_dht_meta_result(&msg)?;
                }
                P2pProtocol::PublishDhtMeta(msg) => {
                    self.priv_handle_publish_dht_meta(&msg)?;
                }
                _ => (),
            }
        }
        Ok(())
    }

    // -- private -- //

    /// send a message to the appropriate channel based on dna_hash::agent_id
    fn priv_send_one(&mut self, dna_hash: &str, agent_id: &str, data: Protocol) -> NetResult<()> {
        if let Some(sender) = self.senders.get_mut(&cat_dna_agent(dna_hash, agent_id)) {
            sender.send(data)?;
        }
        Ok(())
    }

    /// send a message to all nodes connected with this dna hash
    fn priv_send_all(&mut self, dna_hash: &str, data: Protocol) -> NetResult<()> {
        if let Some(arr) = self.senders_by_dna.get_mut(dna_hash) {
            for val in arr.iter_mut() {
                (*val).send(data.clone())?;
            }
        }
        Ok(())
    }

    /// we received a SendMessage message...
    /// normally this would travel over the network, then
    /// show up as a HandleSend message, fabricate that message && deliver
    fn priv_handle_send(&mut self, msg: &MessageData) -> NetResult<()> {
        self.priv_send_one(
            &msg.dna_hash,
            &msg.to_agent_id,
            P2pProtocol::HandleSend(msg.clone()).into(),
        )?;
        Ok(())
    }

    /// we received a SendResult message...
    /// normally this would travel over the network, then
    /// show up as a SendResult message, fabricate that message && deliver
    fn priv_handle_send_result(&mut self, msg: &MessageData) -> NetResult<()> {
        self.priv_send_one(
            &msg.dna_hash,
            &msg.to_agent_id,
            P2pProtocol::SendResult(msg.clone()).into(),
        )?;
        Ok(())
    }

    /// when someone makes a dht data request,
    /// this mock module routes it to the first node connected on that dna.
    /// this works because we also send store requests to all connected nodes.
    fn priv_handle_get_dht(&mut self, msg: &GetDhtData) -> NetResult<()> {
        match self.senders_by_dna.entry(msg.dna_hash.to_string()) {
            Entry::Occupied(mut e) => {
                if !e.get().is_empty() {
                    let r = &e.get_mut()[0];
                    r.send(P2pProtocol::GetDht(msg.clone()).into())?;
                    return Ok(());
                }
            }
            _ => (),
        };

        self.priv_send_one(
            &msg.dna_hash,
            &msg.from_agent_id,
            P2pProtocol::FailureResult(FailureResultData {
                msg_id: msg.msg_id.clone(),
                dna_hash: msg.dna_hash.clone(),
                to_agent_id: msg.from_agent_id.clone(),
                error_info: json!("could not find nodes handling this dnaHash"),
            })
            .into(),
        )?;

        Ok(())
    }

    /// send back a response to a request for dht data
    fn priv_handle_get_dht_result(&mut self, msg: &DhtData) -> NetResult<()> {
        self.priv_send_one(
            &msg.dna_hash,
            &msg.agent_id,
            P2pProtocol::GetDhtResult(msg.clone()).into(),
        )?;
        Ok(())
    }

    /// on publish meta, we send store requests to all nodes connected on this dna
    fn priv_handle_publish_dht(&mut self, msg: &DhtData) -> NetResult<()> {
        self.priv_send_all(&msg.dna_hash, P2pProtocol::StoreDht(msg.clone()).into())?;
        Ok(())
    }

    /// when someone makes a dht meta data request,
    /// this mock module routes it to the first node connected on that dna.
    /// this works because we also send store requests to all connected nodes.
    fn priv_handle_get_dht_meta(&mut self, msg: &GetDhtMetaData) -> NetResult<()> {
        match self.senders_by_dna.entry(msg.dna_hash.to_string()) {
            Entry::Occupied(mut e) => {
                if !e.get().is_empty() {
                    let r = &e.get_mut()[0];
                    r.send(P2pProtocol::GetDhtMeta(msg.clone()).into())?;
                    return Ok(());
                }
            }
            _ => (),
        };

        self.priv_send_one(
            &msg.dna_hash,
            &msg.from_agent_id,
            P2pProtocol::FailureResult(FailureResultData {
                msg_id: msg.msg_id.clone(),
                dna_hash: msg.dna_hash.clone(),
                to_agent_id: msg.from_agent_id.clone(),
                error_info: json!("could not find nodes handling this dnaHash"),
            })
            .into(),
        )?;

        Ok(())
    }

    /// send back a response to a request for dht meta data
    fn priv_handle_get_dht_meta_result(&mut self, msg: &DhtMetaData) -> NetResult<()> {
        self.priv_send_one(
            &msg.dna_hash,
            &msg.agent_id,
            P2pProtocol::GetDhtMetaResult(msg.clone()).into(),
        )?;
        Ok(())
    }

    /// on publish, we send store requests to all nodes connected on this dna
    fn priv_handle_publish_dht_meta(&mut self, msg: &DhtMetaData) -> NetResult<()> {
        self.priv_send_all(
            &msg.dna_hash,
            P2pProtocol::StoreDhtMeta(msg.clone()).into(),
        )?;
        Ok(())
    }
}

/// this is the actual memory space for our mock singleton
lazy_static! {
    static ref MOCK: Mutex<MockSingleton> = Mutex::new(MockSingleton::new());
}

/// make fetching the singleton a little easier
fn get_mock<'a>() -> NetResult<MutexGuard<'a, MockSingleton>> {
    match MOCK.lock() {
        Ok(s) => Ok(s),
        Err(_) => bail!("mock singleton mutex fail"),
    }
}

/// a p2p worker for mocking in-memory scenario tests
pub struct MockWorker {
    handler: NetHandler,
    mock_msgs: Vec<mpsc::Receiver<Protocol>>,
}

impl NetWorker for MockWorker {
    /// stop the net worker
    fn stop(self: Box<Self>) -> NetResult<()> {
        Ok(())
    }

    /// we got a message from holochain core
    /// forward to our mock singleton
    fn receive(&mut self, data: Protocol) -> NetResult<()> {
        let mut mock = get_mock()?;

        if let Ok(wrap) = P2pProtocol::try_from(&data) {
            if let P2pProtocol::TrackApp(app) = wrap {
                let (tx, rx) = mpsc::channel();
                self.mock_msgs.push(rx);
                mock.register(&app.dna_hash, &app.agent_id, tx)?;
                return Ok(());
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
}

impl MockWorker {
    /// create a new mock worker... no configuration required
    pub fn new(handler: NetHandler) -> NetResult<Self> {
        Ok(MockWorker {
            handler,
            mock_msgs: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use holochain_net_connection::protocol_wrapper::{SuccessResultData, TrackAppData};

    static DNA_HASH: &'static str = "blabladnahash";
    static AGENT_ID_1: &'static str = "agent-hash-test-1";
    static AGENT_ID_2: &'static str = "agent-hash-test-2";

    #[test]
    #[cfg_attr(tarpaulin, skip)]
    fn it_mock_networker_flow() {
        // -- setup client 1 -- //

        let (handler_send_1, handler_recv_1) = mpsc::channel::<Protocol>();

        let mut cli1 = Box::new(
            MockWorker::new(Box::new(move |r| {
                handler_send_1.send(r?)?;
                Ok(())
            }))
            .unwrap(),
        );

        cli1.receive(
            P2pProtocol::TrackApp(TrackAppData {
                dna_hash: DNA_HASH.to_string(),
                agent_id: AGENT_ID_1.to_string(),
            })
            .into(),
        )
        .unwrap();

        // -- setup client 2 -- //

        let (handler_send_2, handler_recv_2) = mpsc::channel::<Protocol>();

        let mut cli2 = Box::new(
            MockWorker::new(Box::new(move |r| {
                handler_send_2.send(r?)?;
                Ok(())
            }))
            .unwrap(),
        );

        cli2.receive(
            P2pProtocol::TrackApp(TrackAppData {
                dna_hash: DNA_HASH.to_string(),
                agent_id: AGENT_ID_2.to_string(),
            })
            .into(),
        )
        .unwrap();

        // -- node 2 node / send / receive -- //

        cli1.receive(
            P2pProtocol::SendMessage(MessageData {
                dna_hash: DNA_HASH.to_string(),
                to_agent_id: AGENT_ID_2.to_string(),
                from_agent_id: AGENT_ID_1.to_string(),
                msg_id: "yada".to_string(),
                data: json!("hello"),
            })
            .into(),
        )
        .unwrap();

        cli2.tick().unwrap();

        let res = P2pProtocol::try_from(handler_recv_2.recv().unwrap()).unwrap();

        if let P2pProtocol::HandleSend(msg) = res {
            cli2.receive(
                P2pProtocol::HandleSendResult(MessageData {
                    dna_hash: msg.dna_hash,
                    to_agent_id: msg.from_agent_id,
                    from_agent_id: AGENT_ID_2.to_string(),
                    msg_id: msg.msg_id,
                    data: json!(format!("echo: {}", msg.data.to_string())),
                })
                .into(),
            )
            .unwrap();
        } else {
            panic!("bad msg");
        }

        cli1.tick().unwrap();

        let res = P2pProtocol::try_from(handler_recv_1.recv().unwrap()).unwrap();

        if let P2pProtocol::SendResult(msg) = res {
            assert_eq!("\"echo: \\\"hello\\\"\"".to_string(), msg.data.to_string());
        } else {
            panic!("bad msg");
        }

        // -- dht get -- //

        cli2.receive(
            P2pProtocol::GetDht(GetDhtData {
                msg_id: "yada".to_string(),
                dna_hash: DNA_HASH.to_string(),
                from_agent_id: AGENT_ID_2.to_string(),
                address: "hello".to_string(),
            })
            .into(),
        )
        .unwrap();

        cli1.tick().unwrap();

        let res = P2pProtocol::try_from(handler_recv_1.recv().unwrap()).unwrap();

        if let P2pProtocol::GetDht(msg) = res {
            cli1.receive(
                P2pProtocol::GetDhtResult(DhtData {
                    msg_id: msg.msg_id.clone(),
                    dna_hash: msg.dna_hash.clone(),
                    agent_id: msg.from_agent_id.clone(),
                    address: msg.address.clone(),
                    content: json!(format!("data-for: {}", msg.address)),
                })
                .into(),
            )
            .unwrap();
        } else {
            panic!("bad msg");
        }

        cli2.tick().unwrap();

        let res = P2pProtocol::try_from(handler_recv_2.recv().unwrap()).unwrap();

        if let P2pProtocol::GetDhtResult(msg) = res {
            assert_eq!("\"data-for: hello\"".to_string(), msg.content.to_string());
        } else {
            panic!("bad msg");
        }

        // -- dht publish / store -- //

        cli2.receive(
            P2pProtocol::PublishDht(DhtData {
                msg_id: "yada".to_string(),
                dna_hash: DNA_HASH.to_string(),
                agent_id: AGENT_ID_2.to_string(),
                address: "hello".to_string(),
                content: json!("test-data"),
            })
            .into(),
        )
        .unwrap();

        cli1.tick().unwrap();
        cli2.tick().unwrap();

        let res1 = P2pProtocol::try_from(handler_recv_1.recv().unwrap()).unwrap();
        let res2 = P2pProtocol::try_from(handler_recv_2.recv().unwrap()).unwrap();

        assert_eq!(res1, res2);

        if let P2pProtocol::StoreDht(msg) = res1 {
            cli1.receive(
                P2pProtocol::SuccessResult(SuccessResultData {
                    msg_id: msg.msg_id.clone(),
                    dna_hash: msg.dna_hash.clone(),
                    to_agent_id: msg.agent_id.clone(),
                    success_info: json!("signature here"),
                })
                .into(),
            )
            .unwrap();
        } else {
            panic!("bad msg");
        }

        cli2.tick().unwrap();
        let res = P2pProtocol::try_from(handler_recv_2.recv().unwrap()).unwrap();

        if let P2pProtocol::SuccessResult(msg) = res {
            assert_eq!("\"signature here\"", &msg.success_info.to_string())
        } else {
            panic!("bad msg");
        }

        // -- dht meta get -- //

        cli2.receive(
            P2pProtocol::GetDhtMeta(GetDhtMetaData {
                msg_id: "yada".to_string(),
                dna_hash: DNA_HASH.to_string(),
                from_agent_id: AGENT_ID_2.to_string(),
                address: "hello".to_string(),
                attribute: "link:test".to_string(),
            })
            .into(),
        )
        .unwrap();

        cli1.tick().unwrap();

        let res = P2pProtocol::try_from(handler_recv_1.recv().unwrap()).unwrap();

        if let P2pProtocol::GetDhtMeta(msg) = res {
            cli1.receive(
                P2pProtocol::GetDhtMetaResult(DhtMetaData {
                    msg_id: msg.msg_id.clone(),
                    dna_hash: msg.dna_hash.clone(),
                    agent_id: msg.from_agent_id.clone(),
                    address: msg.address.clone(),
                    attribute: msg.attribute.clone(),
                    content: json!(format!("meta-data-for: {}", msg.address)),
                })
                .into(),
            )
            .unwrap();
        } else {
            panic!("bad msg");
        }

        cli2.tick().unwrap();

        let res = P2pProtocol::try_from(handler_recv_2.recv().unwrap()).unwrap();

        if let P2pProtocol::GetDhtMetaResult(msg) = res {
            assert_eq!(
                "\"meta-data-for: hello\"".to_string(),
                msg.content.to_string()
            );
        } else {
            panic!("bad msg");
        }

        // -- dht meta publish / store -- //

        cli2.receive(
            P2pProtocol::PublishDhtMeta(DhtMetaData {
                msg_id: "yada".to_string(),
                dna_hash: DNA_HASH.to_string(),
                agent_id: AGENT_ID_2.to_string(),
                address: "hello".to_string(),
                attribute: "link:test".to_string(),
                content: json!("test-data"),
            })
            .into(),
        )
        .unwrap();

        cli1.tick().unwrap();
        cli2.tick().unwrap();

        let res1 = P2pProtocol::try_from(handler_recv_1.recv().unwrap()).unwrap();
        let res2 = P2pProtocol::try_from(handler_recv_2.recv().unwrap()).unwrap();

        assert_eq!(res1, res2);

        if let P2pProtocol::StoreDhtMeta(msg) = res1 {
            cli1.receive(
                P2pProtocol::SuccessResult(SuccessResultData {
                    msg_id: msg.msg_id.clone(),
                    dna_hash: msg.dna_hash.clone(),
                    to_agent_id: msg.agent_id.clone(),
                    success_info: json!("signature here"),
                })
                .into(),
            )
            .unwrap();
        } else {
            panic!("bad msg");
        }

        cli2.tick().unwrap();
        let res = P2pProtocol::try_from(handler_recv_2.recv().unwrap()).unwrap();

        if let P2pProtocol::SuccessResult(msg) = res {
            assert_eq!("\"signature here\"", &msg.success_info.to_string())
        } else {
            panic!("bad msg");
        }

        // -- cleanup -- //

        cli1.stop().unwrap();
        cli2.stop().unwrap();
    }
}
