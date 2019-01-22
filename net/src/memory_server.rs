//! provides fake in-memory p2p worker for use in scenario testing

use holochain_core_types::cas::content::Address;
use holochain_net_connection::{
    json_protocol::{
        DhtData, DhtMetaData, FailureResultData, FetchDhtData, FetchDhtMetaData, JsonProtocol,
        MessageData, PeerData, HandleDhtResultData, HandleDhtMetaResultData,
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
    // Keep track of connected clients
    client_count: usize,
}

impl InMemoryServer {
    /// create a new in-memory network server
    pub fn new(name: String) -> Self {
        //println!("NEW InMemoryServer '{}'", name.clone());
        Self {
            senders: HashMap::new(),
            senders_by_dna: HashMap::new(),
            name,
            client_count: 0,
        }
    }

    /// A client clocks in on this server
    pub fn clock_in(&mut self) {
        // Debugging code (do not remove)
        //println!("+++ InMemoryServer '{}' clock_in", self.name.clone());
        self.client_count += 1;
    }

    /// A client clocks out of this server.
    /// If there is no clients left. Clear all the channels.
    pub fn clock_out(&mut self) {
        // Debugging code (do not remove)
        //println!("--- InMemoryServer '{}' clock_out", self.name.clone());
        assert!(self.client_count > 0);
        self.client_count -= 1;
        if self.client_count == 0 {
            //println!("--- InMemoryServer '{}' CLEAR CHANNELS", self.name.clone());
            self.senders.clear();
            self.senders_by_dna.clear();
        }
    }

    //
    pub fn mock_send_one(
        &mut self,
        dna_address: &Address,
        agent_id: &str,
        data: Protocol,
    ) -> NetResult<()> {
        self.priv_send_one(dna_address, agent_id, data)
    }

    //
    pub fn mock_send_all(
        &mut self,
        dna_address: &Address,
        data: Protocol,
    ) -> NetResult<()> {
        self.priv_send_all(dna_address, data)
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

    /// process a message sent by a node to the "network"
    pub fn serve(&mut self, data: Protocol) -> NetResult<()> {
        // Debugging code (do not remove)
        //        println!(
        //            ">>>> InMemoryServer '{}' recv: {:?}",
        //            self.name.clone(),
        //            data
        //        );
        if let Ok(json_msg) = JsonProtocol::try_from(&data) {
            match json_msg {
                JsonProtocol::TrackDna(msg) => {
                    // Notify all Peers connected to this DNA of a new Peer connection.
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
                    self.priv_serve_SendMessage(&msg)?;
                }
                JsonProtocol::HandleSendMessageResult(msg) => {
                    self.priv_serve_HandleSendMessageResult(&msg)?;
                }
                JsonProtocol::SuccessResult(msg) => {
                    // Relay directly the SuccessResult message
                    self.priv_send_one(
                        &msg.dna_address,
                        &msg.to_agent_id,
                        JsonProtocol::SuccessResult(msg.clone()).into(),
                    )?;
                }
                JsonProtocol::FailureResult(msg) => {
                    // Relay directly the FailureResult message
                    self.priv_send_one(
                        &msg.dna_address,
                        &msg.to_agent_id,
                        JsonProtocol::FailureResult(msg.clone()).into(),
                    )?;
                }
                JsonProtocol::FetchDhtData(msg) => {
                    self.priv_serve_FetchDhtData(&msg)?;
                }
                JsonProtocol::HandleFetchDhtDataResult(msg) => {
                    self.priv_serve_HandleFetchDhtDataResult(&msg)?;
                }

                JsonProtocol::PublishDhtData(msg) => {
                    self.priv_serve_PublishDhtData(&msg)?;
                }

                JsonProtocol::FetchDhtMeta(msg) => {
                    self.priv_serve_fetch_dht_meta(&msg)?;
                }
                JsonProtocol::HandleFetchDhtMetaResult(msg) => {
                    self.priv_serve_HandleFetchDhtMetaResult(&msg)?;
                }

                JsonProtocol::PublishDhtMeta(msg) => {
                    self.priv_serve_PublishDhtMeta(&msg)?;
                }
                _ => (),
            }
        }
        Ok(())
    }

    // -- private -- //

    /// send a message to the appropriate channel based on dna_address::to_agent_id
    fn priv_send_one(
        &mut self,
        dna_address: &Address,
        to_agent_id: &str,
        data: Protocol,
    ) -> NetResult<()> {
        let name = cat_dna_agent(dna_address, to_agent_id);
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
        //        println!(
        //            "<<<< InMemoryServer '{}' send: {:?}",
        //            self.name.clone(),
        //            data
        //        );
        sender.send(data)?;
        Ok(())
    }

    /// send a message to all nodes connected with this dna address
    fn priv_send_all(&mut self, dna_address: &Address, data: Protocol) -> NetResult<()> {
        if let Some(arr) = self.senders_by_dna.get_mut(dna_address) {
            // Debugging code (do not remove)
            //            println!(
            //                "<<<< InMemoryServer '{}' send all: {:?} ({})",
            //                self.name.clone(),
            //                data.clone(),
            //                dna_address.clone()
            //            );
            for val in arr.iter_mut() {
                (*val).send(data.clone())?;
            }
        }
        Ok(())
    }

    // -- serve Message -- //

    /// we received a SendMessage message...
    /// normally this would travel over the network, then
    /// show up as a HandleSend message on the receiving agent
    /// Fabricate that message and deliver it to the receiving agent
    fn priv_serve_SendMessage(&mut self, msg: &MessageData) -> NetResult<()> {
        self.priv_send_one(
            &msg.dna_address,
            &msg.to_agent_id,
            JsonProtocol::HandleSendMessage(msg.clone()).into(),
        )?;
        Ok(())
    }

    /// we received a HandleSendMessageResult message...
    /// normally this would travel over the network, then
    /// show up as a SendMessageResult message to the initial sender.
    /// Fabricate that message and deliver it to the initial sender.
    fn priv_serve_HandleSendMessageResult(&mut self, msg: &MessageData) -> NetResult<()> {
        self.priv_send_one(
            &msg.dna_address,
            &msg.to_agent_id,
            JsonProtocol::SendMessageResult(msg.clone()).into(),
        )?;
        Ok(())
    }


    // -- serve DHT data -- //

    /// on publish, we send store requests to all nodes connected on this dna
    fn priv_serve_PublishDhtData(&mut self, msg: &DhtData) -> NetResult<()> {
        self.priv_send_all(
            &msg.dna_address,
            JsonProtocol::HandleStoreDhtData(msg.clone()).into(),
        )?;
        Ok(())
    }

    /// when someone makes a dht data request,
    /// this in-memory module routes it to the first node connected on that dna.
    /// this works because we send store requests to all connected nodes.
    /// If there is no other node for this DNA, send a FailureResult.
    fn priv_serve_FetchDhtData(&mut self, msg: &FetchDhtData) -> NetResult<()> {
        // Find other node and forward request
        match self.senders_by_dna.entry(msg.dna_address.to_owned()) {
            Entry::Occupied(mut e) => {
                if !e.get().is_empty() {
                    let r = &e.get_mut()[0];
                    // Debugging code (do not remove)
                    //println!("<<<< InMemoryServer '{}' send: {:?}", self.name.clone(), msg.clone());
                    r.send(JsonProtocol::HandleFetchDhtData(msg.clone()).into())?;
                    return Ok(());
                }
            }
            _ => (),
        };
        // no other node found, send a FailureResult.
        self.priv_send_one(
            &msg.dna_address,
            &msg.requester_agent_id,
            JsonProtocol::FailureResult(FailureResultData {
                msg_id: msg.request_id.clone(),
                dna_address: msg.dna_address.clone(),
                to_agent_id: msg.requester_agent_id.clone(),
                error_info: json!("could not find nodes handling this dnaAddress"),
            })
            .into(),
        )?;
        // Done
        Ok(())
    }

    /// send back a response to a request for dht data
    fn priv_serve_HandleFetchDhtDataResult(&mut self, msg: &HandleDhtResultData) -> NetResult<()> {
        self.priv_send_one(
            &msg.dna_address,
            &msg.requester_agent_id,
            JsonProtocol::FetchDhtDataResult(msg.clone()).into(),
        )?;
        Ok(())
    }

    // -- serve DHT metadata -- //

    /// on publish, we send store requests to all nodes connected on this dna
    fn priv_serve_PublishDhtMeta(&mut self, msg: &DhtMetaData) -> NetResult<()> {
        self.priv_send_all(
            &msg.dna_address,
            JsonProtocol::HandleStoreDhtMeta(msg.clone()).into(),
        )?;
        Ok(())
    }

    /// when someone makes a dht meta data request,
    /// this in-memory module routes it to the first node connected on that dna.
    /// this works because we also send store requests to all connected nodes.
    fn priv_serve_fetch_dht_meta(&mut self, msg: &FetchDhtMetaData) -> NetResult<()> {
        match self.senders_by_dna.entry(msg.dna_address.to_owned()) {
            Entry::Occupied(mut e) => {
                if !e.get().is_empty() {
                    let r = &e.get_mut()[0];
                    r.send(JsonProtocol::HandleFetchDhtMeta(msg.clone()).into())?;
                    return Ok(());
                }
            }
            _ => (),
        };
        // no other node found, send a FailureResult.
        self.priv_send_one(
            &msg.dna_address,
            &msg.requester_agent_id,
            JsonProtocol::FailureResult(FailureResultData {
                msg_id: msg.request_id.clone(),
                dna_address: msg.dna_address.clone(),
                to_agent_id: msg.requester_agent_id.clone(),
                error_info: json!("could not find nodes handling this dnaAddress"),
            })
            .into(),
        )?;
        // Done
        Ok(())
    }

    /// send back a response to a request for dht meta data
    fn priv_serve_HandleFetchDhtMetaResult(&mut self, msg: &HandleDhtMetaResultData) -> NetResult<()> {
        self.priv_send_one(
            &msg.dna_address,
            &msg.requester_agent_id,
            JsonProtocol::FetchDhtMetaResult(msg.clone()).into(),
        )?;
        Ok(())
    }
}
