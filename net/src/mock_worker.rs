//! provides fake in-memory p2p worker for use in scenario testing

use holochain_net_connection::{
    net_connection::{NetHandler, NetWorker},
    protocol::Protocol,
    protocol_wrapper::{MessageData, ProtocolWrapper},
    NetResult,
};

use std::{
    collections::HashMap,
    convert::TryFrom,
    sync::{mpsc, Mutex, MutexGuard},
};

/// hash connections by dna::agent_id
fn cat_dna_agent(dna_hash: &str, agent_id: &str) -> String {
    format!("{}::{}", dna_hash, agent_id)
}

/// a lazy_static! singleton for routing messages in-memory
struct MockSingleton {
    pub access_count: u64,
    senders: HashMap<String, mpsc::Sender<Protocol>>,
}

impl MockSingleton {
    /// create a new mock singleton
    pub fn new() -> Self {
        Self {
            access_count: 0,
            senders: HashMap::new(),
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
            .insert(cat_dna_agent(dna_hash, agent_id), sender);
        Ok(())
    }

    /// process a message
    pub fn handle(&mut self, data: Protocol) -> NetResult<()> {
        if let Ok(wrap) = ProtocolWrapper::try_from(&data) {
            match wrap {
                ProtocolWrapper::SendMessage(msg) => {
                    self.priv_handle_send(&msg)?;
                }
                ProtocolWrapper::HandleSendResult(msg) => {
                    self.priv_handle_send_result(&msg)?;
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

    /// we received a SendMessage message...
    /// normally this would travel over the network, then
    /// show up as a HandleSend message, fabricate that message && deliver
    fn priv_handle_send(&mut self, msg: &MessageData) -> NetResult<()> {
        self.priv_send_one(
            &msg.dna_hash,
            &msg.to_agent_id,
            ProtocolWrapper::HandleSend(msg.clone()).into(),
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
            ProtocolWrapper::SendResult(msg.clone()).into(),
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

        if let Ok(wrap) = ProtocolWrapper::try_from(&data) {
            if let ProtocolWrapper::TrackApp(app) = wrap {
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

        let mut mock = get_mock()?;
        mock.access_count += 1;
        println!("mock tick ({})", mock.access_count);

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

    use holochain_net_connection::protocol_wrapper::TrackAppData;

    static DNA_HASH: &'static str = "blabladnahash";
    static AGENT_ID_1: &'static str = "agent-hash-test-1";
    static AGENT_ID_2: &'static str = "agent-hash-test-2";

    #[test]
    #[cfg_attr(tarpaulin, skip)]
    fn it_mock_networker_flow() {
        let (handler_send_1, handler_recv_1) = mpsc::channel::<Protocol>();
        let (handler_send_2, handler_recv_2) = mpsc::channel::<Protocol>();

        let mut cli1 = Box::new(
            MockWorker::new(Box::new(move |r| {
                handler_send_1.send(r?)?;
                Ok(())
            })).unwrap(),
        );

        cli1.receive(
            ProtocolWrapper::TrackApp(TrackAppData {
                dna_hash: DNA_HASH.to_string(),
                agent_id: AGENT_ID_1.to_string(),
            }).into(),
        ).unwrap();

        let mut cli2 = Box::new(
            MockWorker::new(Box::new(move |r| {
                handler_send_2.send(r?)?;
                Ok(())
            })).unwrap(),
        );

        cli2.receive(
            ProtocolWrapper::TrackApp(TrackAppData {
                dna_hash: DNA_HASH.to_string(),
                agent_id: AGENT_ID_2.to_string(),
            }).into(),
        ).unwrap();

        cli1.receive(
            ProtocolWrapper::SendMessage(MessageData {
                dna_hash: DNA_HASH.to_string(),
                to_agent_id: AGENT_ID_2.to_string(),
                from_agent_id: AGENT_ID_1.to_string(),
                msg_id: "yada".to_string(),
                data: json!("hello"),
            }).into(),
        ).unwrap();

        cli2.tick().unwrap();

        let res = ProtocolWrapper::try_from(handler_recv_2.recv().unwrap()).unwrap();

        if let ProtocolWrapper::HandleSend(msg) = res {
            cli2.receive(
                ProtocolWrapper::HandleSendResult(MessageData {
                    dna_hash: msg.dna_hash,
                    to_agent_id: msg.from_agent_id,
                    from_agent_id: AGENT_ID_2.to_string(),
                    msg_id: msg.msg_id,
                    data: json!(format!("echo: {}", msg.data.to_string())),
                }).into(),
            ).unwrap();
        } else {
            panic!("bad msg");
        }

        cli1.tick().unwrap();

        let res = ProtocolWrapper::try_from(handler_recv_1.recv().unwrap()).unwrap();

        if let ProtocolWrapper::SendResult(msg) = res {
            assert_eq!("\"echo: \\\"hello\\\"\"".to_string(), msg.data.to_string());
        } else {
            panic!("bad msg");
        }

        cli1.stop().unwrap();
        cli2.stop().unwrap();
    }
}
