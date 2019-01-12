#![allow(non_snake_case)]

//! provides fake in-memory p2p worker for use in scenario testing

use holochain_core_types::cas::content::Address;
use holochain_net_connection::{
    protocol::Protocol,
    protocol_wrapper::{
        DhtData, DhtMetaData, FailureResultData, GetDhtData, GetDhtMetaData, MessageData,
        ProtocolWrapper,
    },
    NetResult,
};

use std::{
    collections::{hash_map::Entry, HashMap},
    convert::TryFrom,
    sync::{mpsc, Mutex, MutexGuard},
};


/// hash connections by dna::agent_id
fn cat_dna_agent(dna_address: &Address, agent_id: &str) -> String {
    format!("{}::{}", dna_address, agent_id)
}

/// a lazy_static! singleton for routing messages in-memory
pub struct MockSingleton {
    // keep track of senders by `dna_address::agent_id`
    senders: HashMap<String, mpsc::Sender<Protocol>>,
    // keep track of senders as arrays by dna_address
    senders_by_dna: HashMap<Address, Vec<mpsc::Sender<Protocol>>>,
}

impl MockSingleton {
    /// create a new mock singleton
    pub fn new() -> Self {
        Self {
            senders: HashMap::new(),
            senders_by_dna: HashMap::new(),
        }
    }

    /// register a data handler (from a MockWorker) (for message routing)
    pub fn register(
        &mut self,
        dna_address: &Address,
        agent_id: &str,
        sender: mpsc::Sender<Protocol>,
    ) -> NetResult<()> {
        self.senders
            .insert(cat_dna_agent(dna_address, agent_id), sender.clone());
        match self.senders_by_dna.entry(dna_address.to_owned()) {
            Entry::Occupied(mut e) => {
                e.get_mut().push(sender.clone());
            }
            Entry::Vacant(e) => {
                e.insert(vec![sender.clone()]);
            }
        };
        Ok(())
    }

    /// process an incoming message
    pub fn handle(&mut self, data: Protocol) -> NetResult<()> {
        if let Ok(wrap) = ProtocolWrapper::try_from(&data) {
            match wrap {
                ProtocolWrapper::GenericMessage(msg) => {
                    self.priv_handle_GenericMessage(&msg)?;
                }
                ProtocolWrapper::HandleGenericMessageResponse(msg) => {
                    self.priv_handle_HandleGenericMessageResponse(&msg)?;
                }
                ProtocolWrapper::SuccessResult(msg) => {
                    self.priv_send_one(
                        &msg.dna_address,
                        &msg.to_agent_id,
                        ProtocolWrapper::SuccessResult(msg.clone()).into(),
                    )?;
                }
                ProtocolWrapper::FailureResult(msg) => {
                    self.priv_send_one(
                        &msg.dna_address,
                        &msg.to_agent_id,
                        ProtocolWrapper::FailureResult(msg.clone()).into(),
                    )?;
                }
                ProtocolWrapper::GetDht(msg) => {
                    self.priv_handle_get_dht(&msg)?;
                }
                ProtocolWrapper::GetDhtResult(msg) => {
                    self.priv_handle_get_dht_result(&msg)?;
                }
                ProtocolWrapper::PublishDht(msg) => {
                    self.priv_handle_publish_dht(&msg)?;
                }
                ProtocolWrapper::GetDhtMeta(msg) => {
                    self.priv_handle_get_dht_meta(&msg)?;
                }
                ProtocolWrapper::GetDhtMetaResult(msg) => {
                    self.priv_handle_get_dht_meta_result(&msg)?;
                }
                ProtocolWrapper::PublishDhtMeta(msg) => {
                    self.priv_handle_publish_dht_meta(&msg)?;
                }
                _ => (),
            }
        }
        Ok(())
    }

    // -- private -- //

    /// send a message to the appropriate channel based on 'dna_address::agent_id'
    fn priv_send_one(
        &mut self,
        dna_address: &Address,
        agent_id: &str,
        data: Protocol,
    ) -> NetResult<()> {
        if let Some(sender) = self.senders.get_mut(&cat_dna_agent(dna_address, agent_id)) {
            sender.send(data)?;
        }
        Ok(())
    }

    /// send a message to all nodes connected with this dna address
    fn priv_send_all(&mut self, dna_address: &Address, data: Protocol) -> NetResult<()> {
        if let Some(arr) = self.senders_by_dna.get_mut(dna_address) {
            for val in arr.iter_mut() {
                (*val).send(data.clone())?;
            }
        }
        Ok(())
    }

    /// we received a GenericMessage message...
    /// normally this would travel over the network, then show up as a HandleGenericMessage,
    /// fabricate that message && deliver
    fn priv_handle_GenericMessage(&mut self, msg: &MessageData) -> NetResult<()> {
        println!("*** priv_handle_GenericMessage()");
        self.priv_send_one(
            &msg.dna_address,
            &msg.to_agent_id,
            ProtocolWrapper::HandleGenericMessage(msg.clone()).into(),
        )?;
        Ok(())
    }

    /// we received a HandleGenericMessageResponse message...
    /// normally this would travel over the network,
    /// then show up as a GenericMessageResponse message,
    /// fabricate that message && deliver
    fn priv_handle_HandleGenericMessageResponse(&mut self, msg: &MessageData) -> NetResult<()> {
        self.priv_send_one(
            &msg.dna_address,
            &msg.to_agent_id,
            ProtocolWrapper::GenericMessageResponse(msg.clone()).into(),
        )?;
        Ok(())
    }

    /// when someone makes a dht data request,
    /// this mock module routes it to the first node connected on that dna.
    /// this works because we also send store requests to all connected nodes.
    fn priv_handle_get_dht(&mut self, msg: &GetDhtData) -> NetResult<()> {
        match self.senders_by_dna.entry(msg.dna_address.to_owned()) {
            Entry::Occupied(mut e) => {
                if !e.get().is_empty() {
                    let r = &e.get_mut()[0];
                    r.send(ProtocolWrapper::GetDht(msg.clone()).into())?;
                    return Ok(());
                }
            }
            _ => (),
        };

        self.priv_send_one(
            &msg.dna_address,
            &msg.from_agent_id,
            ProtocolWrapper::FailureResult(FailureResultData {
                msg_id: msg.msg_id.clone(),
                dna_address: msg.dna_address.clone(),
                to_agent_id: msg.from_agent_id.clone(),
                error_info: json!("could not find nodes handling this dnaAddress"),
            })
                .into(),
        )?;

        Ok(())
    }

    /// send back a response to a request for dht data
    fn priv_handle_get_dht_result(&mut self, msg: &DhtData) -> NetResult<()> {
        self.priv_send_one(
            &msg.dna_address,
            &msg.agent_id,
            ProtocolWrapper::GetDhtResult(msg.clone()).into(),
        )?;
        Ok(())
    }

    /// on publish meta, we send store requests to all nodes connected on this dna
    fn priv_handle_publish_dht(&mut self, msg: &DhtData) -> NetResult<()> {
        self.priv_send_all(
            &msg.dna_address,
            ProtocolWrapper::StoreDht(msg.clone()).into(),
        )?;
        Ok(())
    }

    /// when someone makes a dht meta data request,
    /// this mock module routes it to the first node connected on that dna.
    /// this works because we also send store requests to all connected nodes.
    fn priv_handle_get_dht_meta(&mut self, msg: &GetDhtMetaData) -> NetResult<()> {
        match self.senders_by_dna.entry(msg.dna_address.to_owned()) {
            Entry::Occupied(mut e) => {
                if !e.get().is_empty() {
                    let r = &e.get_mut()[0];
                    r.send(ProtocolWrapper::GetDhtMeta(msg.clone()).into())?;
                    return Ok(());
                }
            }
            _ => (),
        };

        self.priv_send_one(
            &msg.dna_address,
            &msg.from_agent_id,
            ProtocolWrapper::FailureResult(FailureResultData {
                msg_id: msg.msg_id.clone(),
                dna_address: msg.dna_address.clone(),
                to_agent_id: msg.from_agent_id.clone(),
                error_info: json!("could not find nodes handling this dnaAddress"),
            })
                .into(),
        )?;

        Ok(())
    }

    /// send back a response to a request for dht meta data
    fn priv_handle_get_dht_meta_result(&mut self, msg: &DhtMetaData) -> NetResult<()> {
        self.priv_send_one(
            &msg.dna_address,
            &msg.agent_id,
            ProtocolWrapper::GetDhtMetaResult(msg.clone()).into(),
        )?;
        Ok(())
    }

    /// on publish, we send store requests to all nodes connected on this dna
    fn priv_handle_publish_dht_meta(&mut self, msg: &DhtMetaData) -> NetResult<()> {
        self.priv_send_all(
            &msg.dna_address,
            ProtocolWrapper::StoreDhtMeta(msg.clone()).into(),
        )?;
        Ok(())
    }
}

/// this is the actual memory space for our mock singleton
lazy_static! {
    static ref MOCK: Mutex<MockSingleton> = Mutex::new(MockSingleton::new());
}

/// make fetching the singleton a little easier
pub fn get_mock_singleton<'a>() -> NetResult<MutexGuard<'a, MockSingleton>> {
    match MOCK.lock() {
        Ok(s) => Ok(s),
        Err(_) => bail!("mock singleton mutex fail"),
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::p2p_config::P2pConfig;

    use holochain_net_connection::protocol_wrapper::{SuccessResultData, TrackAppData};

    fn example_dna_address() -> Address {
        "blabladnaAddress".into()
    }

    static AGENT_ID_1: &'static str = "agent-hash-test-1";
    static AGENT_ID_2: &'static str = "agent-hash-test-2";

    #[test]
    #[cfg_attr(tarpaulin, skip)]
    fn it_mock_networker_flow() {
        // -- setup client 1 -- //

        let config = &JsonString::from(P2pConfig::unique_mock_as_string());

        let (handler_send_1, handler_recv_1) = mpsc::channel::<Protocol>();

        let mut cli1 = Box::new(
            MockWorker::new(
                Box::new(move |r| {
                    handler_send_1.send(r?)?;
                    Ok(())
                }),
                config,
            )
                .unwrap(),
        );

        cli1.receive(
            ProtocolWrapper::TrackApp(TrackAppData {
                dna_address: example_dna_address(),
                agent_id: AGENT_ID_1.to_string(),
            })
                .into(),
        )
            .unwrap();

        // -- setup client 2 -- //

        let (handler_send_2, handler_recv_2) = mpsc::channel::<Protocol>();

        let mut cli2 = Box::new(
            MockWorker::new(
                Box::new(move |r| {
                    handler_send_2.send(r?)?;
                    Ok(())
                }),
                config,
            )
                .unwrap(),
        );

        cli2.receive(
            ProtocolWrapper::TrackApp(TrackAppData {
                dna_address: example_dna_address(),
                agent_id: AGENT_ID_2.to_string(),
            })
                .into(),
        )
            .unwrap();

        // -- node 2 node / send / receive -- //

        cli1.receive(
            ProtocolWrapper::GenericMessage(MessageData {
                dna_address: example_dna_address(),
                to_agent_id: AGENT_ID_2.to_string(),
                from_agent_id: AGENT_ID_1.to_string(),
                msg_id: "yada".to_string(),
                data: json!("hello"),
            })
                .into(),
        )
            .unwrap();

        cli2.tick().unwrap();

        let res = ProtocolWrapper::try_from(handler_recv_2.recv().unwrap()).unwrap();

        if let ProtocolWrapper::HandleGenericMessage(msg) = res {
            cli2.receive(
                ProtocolWrapper::HandleGenericMessageResponse(MessageData {
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
            panic!("bad msg");
        }

        cli1.tick().unwrap();

        let res = ProtocolWrapper::try_from(handler_recv_1.recv().unwrap()).unwrap();

        if let ProtocolWrapper::GenericMessageResponse(msg) = res {
            assert_eq!("\"echo: \\\"hello\\\"\"".to_string(), msg.data.to_string());
        } else {
            panic!("bad msg");
        }

        // -- dht get -- //

        cli2.receive(
            ProtocolWrapper::GetDht(GetDhtData {
                msg_id: "yada".to_string(),
                dna_address: example_dna_address(),
                from_agent_id: AGENT_ID_2.to_string(),
                address: "hello".to_string(),
            })
                .into(),
        )
            .unwrap();

        cli1.tick().unwrap();

        let res = ProtocolWrapper::try_from(handler_recv_1.recv().unwrap()).unwrap();

        if let ProtocolWrapper::GetDht(msg) = res {
            cli1.receive(
                ProtocolWrapper::GetDhtResult(DhtData {
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
            panic!("bad msg");
        }

        cli2.tick().unwrap();

        let res = ProtocolWrapper::try_from(handler_recv_2.recv().unwrap()).unwrap();

        if let ProtocolWrapper::GetDhtResult(msg) = res {
            assert_eq!("\"data-for: hello\"".to_string(), msg.content.to_string());
        } else {
            panic!("bad msg");
        }

        // -- dht publish / store -- //

        cli2.receive(
            ProtocolWrapper::PublishDht(DhtData {
                msg_id: "yada".to_string(),
                dna_address: example_dna_address(),
                agent_id: AGENT_ID_2.to_string(),
                address: "hello".to_string(),
                content: json!("test-data"),
            })
                .into(),
        )
            .unwrap();

        cli1.tick().unwrap();
        cli2.tick().unwrap();

        let res1 = ProtocolWrapper::try_from(handler_recv_1.recv().unwrap()).unwrap();
        let res2 = ProtocolWrapper::try_from(handler_recv_2.recv().unwrap()).unwrap();

        assert_eq!(res1, res2);

        if let ProtocolWrapper::StoreDht(msg) = res1 {
            cli1.receive(
                ProtocolWrapper::SuccessResult(SuccessResultData {
                    msg_id: msg.msg_id.clone(),
                    dna_address: msg.dna_address.clone(),
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
        let res = ProtocolWrapper::try_from(handler_recv_2.recv().unwrap()).unwrap();

        if let ProtocolWrapper::SuccessResult(msg) = res {
            assert_eq!("\"signature here\"", &msg.success_info.to_string())
        } else {
            panic!("bad msg");
        }

        // -- dht meta get -- //

        cli2.receive(
            ProtocolWrapper::GetDhtMeta(GetDhtMetaData {
                msg_id: "yada".to_string(),
                dna_address: example_dna_address(),
                from_agent_id: AGENT_ID_2.to_string(),
                address: "hello".to_string(),
                attribute: "link:test".to_string(),
            })
                .into(),
        )
            .unwrap();

        cli1.tick().unwrap();

        let res = ProtocolWrapper::try_from(handler_recv_1.recv().unwrap()).unwrap();

        if let ProtocolWrapper::GetDhtMeta(msg) = res {
            cli1.receive(
                ProtocolWrapper::GetDhtMetaResult(DhtMetaData {
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
            panic!("bad msg");
        }

        cli2.tick().unwrap();

        let res = ProtocolWrapper::try_from(handler_recv_2.recv().unwrap()).unwrap();

        if let ProtocolWrapper::GetDhtMetaResult(msg) = res {
            assert_eq!(
                "\"meta-data-for: hello\"".to_string(),
                msg.content.to_string()
            );
        } else {
            panic!("bad msg");
        }

        // -- dht meta publish / store -- //

        cli2.receive(
            ProtocolWrapper::PublishDhtMeta(DhtMetaData {
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

        cli1.tick().unwrap();
        cli2.tick().unwrap();

        let res1 = ProtocolWrapper::try_from(handler_recv_1.recv().unwrap()).unwrap();
        let res2 = ProtocolWrapper::try_from(handler_recv_2.recv().unwrap()).unwrap();

        assert_eq!(res1, res2);

        if let ProtocolWrapper::StoreDhtMeta(msg) = res1 {
            cli1.receive(
                ProtocolWrapper::SuccessResult(SuccessResultData {
                    msg_id: msg.msg_id.clone(),
                    dna_address: msg.dna_address.clone(),
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
        let res = ProtocolWrapper::try_from(handler_recv_2.recv().unwrap()).unwrap();

        if let ProtocolWrapper::SuccessResult(msg) = res {
            assert_eq!("\"signature here\"", &msg.success_info.to_string())
        } else {
            panic!("bad msg");
        }

        // -- cleanup -- //

        cli1.stop().unwrap();
        cli2.stop().unwrap();
    }
}
