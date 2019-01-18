//! provides fake in-memory p2p worker for use in scenario testing

use holochain_core_types::cas::content::Address;
use holochain_net_connection::{
    json_protocol::{
        DhtData, DhtMetaData, FailureResultData, GetDhtData, GetDhtMetaData, JsonProtocol,
        MessageData, PeerData,
    },
    protocol::Protocol,
    NetResult,
};
use std::{
    collections::{hash_map::Entry, HashMap},
    convert::TryFrom,
    sync::{mpsc, Mutex, RwLock},
};

type InMemoryServerMap = HashMap<String, Mutex<InMemoryServer>>;

/// this is the actual memory space for our in-memory servers
lazy_static! {
    pub(crate) static ref MEMORY_SERVER_MAP: RwLock<InMemoryServerMap> =
        RwLock::new(HashMap::new());
}

/// hash connections by dna::agent_id
fn cat_dna_agent(dna_address: &Address, agent_id: &str) -> String {
    format!("{}::{}", dna_address, agent_id)
}

/// a lazy_static! singleton for routing messages in-memory
pub(crate) struct InMemoryServer {
    // keep track of senders by `dna_address::agent_id`
    senders: HashMap<String, mpsc::Sender<Protocol>>,
    // keep track of senders as arrays by dna_address
    senders_by_dna: HashMap<Address, Vec<mpsc::Sender<Protocol>>>,
    // Unique identifier
    name: String,
}

impl InMemoryServer {
    /// create a new in-memory network server
    pub fn new(name: String) -> Self {
        Self {
            senders: HashMap::new(),
            senders_by_dna: HashMap::new(),
            name,
        }
    }

    /// register a data handler with the singleton (for message routing)
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
        // Debugging code (do not remove)
        //println!(">>>> InMemoryServer '{}' recv: {:?}", self.name.clone(), data);
        if let Ok(json_msg) = JsonProtocol::try_from(&data) {
            match json_msg {
                JsonProtocol::TrackDna(msg) => {
                    self.priv_send_all(
                        &msg.dna_address.clone(),
                        JsonProtocol::PeerConnected(PeerData {
                            dna_address: msg.dna_address,
                            agent_id: msg.agent_id,
                        })
                        .into(),
                    )?;
                }

                JsonProtocol::SendMessage(msg) => {
                    self.priv_handle_send_message(&msg)?;
                }
                JsonProtocol::HandleSendMessageResult(msg) => {
                    self.priv_handle_handle_send_message_result(&msg)?;
                }
                JsonProtocol::SuccessResult(msg) => {
                    self.priv_send_one(
                        &msg.dna_address,
                        &msg.to_agent_id,
                        JsonProtocol::SuccessResult(msg.clone()).into(),
                    )?;
                }
                JsonProtocol::FailureResult(msg) => {
                    self.priv_send_one(
                        &msg.dna_address,
                        &msg.to_agent_id,
                        JsonProtocol::FailureResult(msg.clone()).into(),
                    )?;
                }
                JsonProtocol::GetDhtData(msg) => {
                    self.priv_handle_get_dht_data(&msg)?;
                }
                JsonProtocol::HandleGetDhtDataResult(msg) => {
                    self.priv_handle_handle_get_dht_data_result(&msg)?;
                }

                JsonProtocol::PublishDhtData(msg) => {
                    self.priv_handle_publish_dht_data(&msg)?;
                }

                JsonProtocol::GetDhtMeta(msg) => {
                    self.priv_handle_get_dht_meta(&msg)?;
                }
                JsonProtocol::HandleGetDhtMetaResult(msg) => {
                    self.priv_handle_handle_get_dht_meta_result(&msg)?;
                }

                JsonProtocol::PublishDhtMeta(msg) => {
                    self.priv_handle_publish_dht_meta(&msg)?;
                }
                _ => (),
            }
        }
        Ok(())
    }

    // -- private -- //

    /// send a message to the appropriate channel based on dna_address::agent_id
    fn priv_send_one(
        &mut self,
        dna_address: &Address,
        agent_id: &str,
        data: Protocol,
    ) -> NetResult<()> {
        let name = cat_dna_agent(dna_address, agent_id);
        let maybe_sender = self.senders.get_mut(&name);
        if maybe_sender.is_none() {
            //println!("#### InMemoryServer '{}' error: No sender channel found", self.name.clone());
            return Err(format_err!(
                "No sender channel found ({})",
                self.name.clone()
            ));
        }
        let sender = maybe_sender.unwrap();
        // Debugging code (do not remove)
        //println!("<<<< InMemoryServer '{}' send: {:?}", self.name.clone(), data);
        sender.send(data)?;
        Ok(())
    }

    /// send a message to all nodes connected with this dna address
    fn priv_send_all(&mut self, dna_address: &Address, data: Protocol) -> NetResult<()> {
        if let Some(arr) = self.senders_by_dna.get_mut(dna_address) {
            // Debugging code (do not remove)
            //println!("<<<< InMemoryServer '{}' send all: {:?} ({})", self.name.clone(), data.clone(), dna_address.clone());
            for val in arr.iter_mut() {
                (*val).send(data.clone())?;
            }
        }
        Ok(())
    }

    /// we received a SendMessage message...
    /// normally this would travel over the network, then
    /// show up as a HandleSend message, fabricate that message && deliver
    fn priv_handle_send_message(&mut self, msg: &MessageData) -> NetResult<()> {
        self.priv_send_one(
            &msg.dna_address,
            &msg.to_agent_id,
            JsonProtocol::HandleSendMessage(msg.clone()).into(),
        )?;
        Ok(())
    }

    /// we received a SendResult message...
    /// normally this would travel over the network, then
    /// show up as a SendResult message, fabricate that message && deliver
    fn priv_handle_handle_send_message_result(&mut self, msg: &MessageData) -> NetResult<()> {
        self.priv_send_one(
            &msg.dna_address,
            &msg.to_agent_id,
            JsonProtocol::SendMessageResult(msg.clone()).into(),
        )?;
        Ok(())
    }

    /// when someone makes a dht data request,
    /// this in-memory module routes it to the first node connected on that dna.
    /// this works because we also send store requests to all connected nodes.
    fn priv_handle_get_dht_data(&mut self, msg: &GetDhtData) -> NetResult<()> {
        match self.senders_by_dna.entry(msg.dna_address.to_owned()) {
            Entry::Occupied(mut e) => {
                if !e.get().is_empty() {
                    let r = &e.get_mut()[0];
                    // Debugging code (do not remove)
                    //println!("<<<< InMemoryServer '{}' send: {:?}", self.name.clone(), msg.clone());
                    r.send(JsonProtocol::HandleGetDhtData(msg.clone()).into())?;
                    return Ok(());
                }
            }
            _ => (),
        };

        self.priv_send_one(
            &msg.dna_address,
            &msg.from_agent_id,
            JsonProtocol::FailureResult(FailureResultData {
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
    fn priv_handle_handle_get_dht_data_result(&mut self, msg: &DhtData) -> NetResult<()> {
        self.priv_send_one(
            &msg.dna_address,
            &msg.agent_id,
            JsonProtocol::GetDhtDataResult(msg.clone()).into(),
        )?;
        Ok(())
    }

    /// on publish meta, we send store requests to all nodes connected on this dna
    fn priv_handle_publish_dht_data(&mut self, msg: &DhtData) -> NetResult<()> {
        self.priv_send_all(
            &msg.dna_address,
            JsonProtocol::HandleStoreDhtData(msg.clone()).into(),
        )?;
        Ok(())
    }

    /// when someone makes a dht meta data request,
    /// this in-memory module routes it to the first node connected on that dna.
    /// this works because we also send store requests to all connected nodes.
    fn priv_handle_get_dht_meta(&mut self, msg: &GetDhtMetaData) -> NetResult<()> {
        match self.senders_by_dna.entry(msg.dna_address.to_owned()) {
            Entry::Occupied(mut e) => {
                if !e.get().is_empty() {
                    let r = &e.get_mut()[0];
                    r.send(JsonProtocol::HandleGetDhtMeta(msg.clone()).into())?;
                    return Ok(());
                }
            }
            _ => (),
        };

        self.priv_send_one(
            &msg.dna_address,
            &msg.from_agent_id,
            JsonProtocol::FailureResult(FailureResultData {
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
    fn priv_handle_handle_get_dht_meta_result(&mut self, msg: &DhtMetaData) -> NetResult<()> {
        self.priv_send_one(
            &msg.dna_address,
            &msg.agent_id,
            JsonProtocol::GetDhtMetaResult(msg.clone()).into(),
        )?;
        Ok(())
    }

    /// on publish, we send store requests to all nodes connected on this dna
    fn priv_handle_publish_dht_meta(&mut self, msg: &DhtMetaData) -> NetResult<()> {
        self.priv_send_all(
            &msg.dna_address,
            JsonProtocol::HandleStoreDhtMeta(msg.clone()).into(),
        )?;
        Ok(())
    }
}
