//! provides in-memory p2p "server" for use in scenario testing
//! the server connects all the memory_workers together, so there is no real gossiping going around.
//! Could have pluggable DHT strategy. Full-sync currently hard-coded: #fulldht

#![allow(non_snake_case)]

use super::memory_book::*;
use crate::{
    connection::{
        json_protocol::{
            DhtMetaData, EntryData, EntryListData, FailureResultData, FetchEntryData,
            FetchEntryResultData, FetchMetaData, FetchMetaResultData, GetListData, JsonProtocol,
            MessageData, MetaListData, PeerData,
        },
        protocol::Protocol,
        NetResult,
    },
    error::NetworkError,
    tweetlog::*,
};
use holochain_core_types::cas::content::Address;
use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    convert::TryFrom,
    sync::{mpsc, Mutex, RwLock},
};

type RequestId = String;

/// Type for holding a map of 'network_name -> InMemoryServer'
type InMemoryServerMap = HashMap<String, Mutex<InMemoryServer>>;

/// this is the actual memory space for our in-memory servers
lazy_static! {
    pub(crate) static ref MEMORY_SERVER_MAP: RwLock<InMemoryServerMap> =
        RwLock::new(HashMap::new());
}

/// a global server for routing messages between nodes in-memory
pub(crate) struct InMemoryServer {
    // keep track of senders by `dna_address::agent_id`
    senders: HashMap<CellId, mpsc::Sender<Protocol>>,
    // keep track of agents by dna_address
    senders_by_dna: HashMap<Address, HashMap<String, mpsc::Sender<Protocol>>>,
    // Unique identifier
    name: String,
    // Keep track of connected clients
    client_count: usize,

    // All published data book: cell_id -> entry_addresses
    published_book: CellBook,
    // All stored data book: cell_id -> entry_addresses
    stored_book: CellBook,

    // Keep track of which DNAs are "tracked"
    trackdna_book: HashSet<CellId>,
    // Keep track of requests authored by the server
    // request_id -> cell_id
    request_book: HashMap<RequestId, CellId>,
    // used for making unique request ids
    request_count: usize,

    // Logger
    log: TweetProxy,
}

/// Books handling
impl InMemoryServer {
    /// generate a new request_id
    fn priv_generate_request_id(&mut self) -> String {
        self.request_count += 1;
        format!("req_{}", self.request_count)
    }

    /// generate a new request, return the request_id  (sugar)
    fn priv_create_request(&mut self, dna_address: &Address, agent_id: &str) -> RequestId {
        let cell_id = into_cell_id(dna_address, agent_id);
        self.priv_create_request_with_cell_id(&cell_id)
    }

    /// generate a new request, return the request_id
    fn priv_create_request_with_cell_id(&mut self, cell_id: &CellId) -> RequestId {
        let req_id = self.priv_generate_request_id();
        self.request_book
            .insert(req_id.clone(), cell_id.to_string());
        req_id
    }

    /// Check if its our own request and return CellId
    fn priv_check_request(&self, request_id: &RequestId) -> Option<&CellId> {
        self.log.t(&format!(
            "---- priv_check_request('{}') in {:?} ?",
            request_id,
            self.request_book.clone(),
        ));
        self.request_book.get(&request_id.clone())
    }

    /// Send all Get*Lists requests to agent
    fn priv_request_all_lists(&mut self, dna_address: &Address, agent_id: &str) {
        // Entry
        // Request this agent's published entries
        let request_id = self.priv_create_request(dna_address, agent_id);
        self.priv_send_one(
            dna_address,
            agent_id,
            JsonProtocol::HandleGetPublishingEntryList(GetListData {
                request_id,
                dna_address: dna_address.clone(),
            })
            .into(),
        )
        .expect("Sending HandleGetPublishingEntryList failed");
        // Request this agent's holding entries
        let request_id = self.priv_create_request(dna_address, agent_id);
        self.priv_send_one(
            dna_address,
            agent_id,
            JsonProtocol::HandleGetHoldingEntryList(GetListData {
                request_id,
                dna_address: dna_address.clone(),
            })
            .into(),
        )
        .expect("Sending HandleGetHoldingEntryList failed");

        // Metadata
        // Request this agent's published metadata
        let request_id = self.priv_create_request(dna_address, agent_id);
        self.priv_send_one(
            dna_address,
            agent_id,
            JsonProtocol::HandleGetPublishingMetaList(GetListData {
                request_id,
                dna_address: dna_address.clone(),
            })
            .into(),
        )
        .expect("Sending HandleGetPublishingMetaList failed");
        // Request this agent's holding metadata
        let request_id = self.priv_create_request(dna_address, agent_id);
        self.priv_send_one(
            dna_address,
            agent_id,
            JsonProtocol::HandleGetHoldingMetaList(GetListData {
                request_id,
                dna_address: dna_address.clone(),
            })
            .into(),
        )
        .expect("Sending HandleGetHoldingMetaList failed");
    }
}

/// Public API
impl InMemoryServer {
    /// create a new in-memory network server
    pub fn new(name: String) -> Self {
        Self {
            name,
            senders: HashMap::new(),
            senders_by_dna: HashMap::new(),
            client_count: 0,
            request_book: HashMap::new(),
            published_book: HashMap::new(),
            stored_book: HashMap::new(),
            request_count: 0,
            trackdna_book: HashSet::new(),
            log: TweetProxy::new("memory_server"),
        }
    }

    /// A client clocks in on this server
    pub fn clock_in(&mut self) {
        self.log
            .t(&format!("+++ '{}' clock_in()", self.name.clone()));
        self.client_count += 1;
    }

    /// A client clocks out of this server.
    /// If there is no clients left. Clear all the channels.
    pub fn clock_out(&mut self) {
        self.log
            .t(&format!("--- '{}' clock_out", self.name.clone()));
        assert!(self.client_count > 0);
        self.client_count -= 1;
        if self.client_count == 0 {
            self.log
                .t(&format!("--- '{}' CLEAR CHANNELS", self.name.clone()));
            self.senders.clear();
            self.senders_by_dna.clear();
        }
    }

    /// register a cell's handler with the server (for message routing)
    pub fn register_cell(
        &mut self,
        dna_address: &Address,
        agent_id: &str,
        sender: mpsc::Sender<Protocol>,
    ) -> NetResult<()> {
        self.senders
            .insert(into_cell_id(dna_address, agent_id), sender.clone());
        match self.senders_by_dna.entry(dna_address.to_owned()) {
            Entry::Occupied(mut e) => {
                e.get_mut().insert(agent_id.to_string(), sender.clone());
            }
            Entry::Vacant(e) => {
                let mut map = HashMap::new();
                map.insert(agent_id.to_string(), sender.clone());
                e.insert(map);
            }
        };
        Ok(())
    }

    /// unregister a cell's handler with the server (for message routing)
    pub fn unregister_cell(&mut self, dna_address: &Address, agent_id: &str) {
        let cell_id = into_cell_id(dna_address, agent_id);
        self.log.d(&format!("unregistering '{}'", cell_id));
        let maybe_sender = self.senders.remove(&cell_id);
        if maybe_sender.is_none() {
            return;
        }
        match self.senders_by_dna.entry(dna_address.to_owned()) {
            Entry::Occupied(mut senders) => {
                senders.get_mut().remove(agent_id.clone());
            }
            Entry::Vacant(_) => unreachable!(),
        };
        self.log.d(&format!("unregistering '{}' DONE", cell_id));
    }

    /// process a message sent by a node to the "network"
    pub fn serve(&mut self, data: Protocol) -> NetResult<()> {
        self.log
            .d(&format!(">>>> '{}' recv: {:?}", self.name.clone(), data));
        // serve only JsonProtocol
        let maybe_json_msg = JsonProtocol::try_from(&data);
        if let Err(_) = maybe_json_msg {
            return Ok(());
        };
        // Note: use same order as the enum
        match maybe_json_msg.unwrap() {
            JsonProtocol::SuccessResult(msg) => {
                // Check if agent is tracking the dna
                let is_tracked =
                    self.priv_check_or_fail(&msg.dna_address, &msg.to_agent_id, None)?;
                if !is_tracked {
                    return Ok(());
                }
                // Relay directly the SuccessResult message
                self.priv_send_one(
                    &msg.dna_address,
                    &msg.to_agent_id,
                    JsonProtocol::SuccessResult(msg.clone()).into(),
                )?;
            }
            JsonProtocol::FailureResult(msg) => {
                // Check if agent is tracking the dna
                let is_tracked =
                    self.priv_check_or_fail(&msg.dna_address, &msg.to_agent_id, None)?;
                if !is_tracked {
                    return Ok(());
                }
                // Check if its a response to our own request
                {
                    let maybe_cell_id = self.priv_check_request(&msg.request_id);
                    if let Some(_) = maybe_cell_id {
                        self.log.d(&format!(
                            "---- '{}' internal request failed: {:?}",
                            self.name.clone(),
                            msg.clone(),
                        ));
                        return Ok(());
                    }
                }
                // If not, relay the FailureResult message to receipient
                self.priv_send_one(
                    &msg.dna_address,
                    &msg.to_agent_id,
                    JsonProtocol::FailureResult(msg.clone()).into(),
                )?;
            }
            JsonProtocol::TrackDna(msg) => {
                // Check if we are already tracking this dna for this agent
                let cell_id = into_cell_id(&msg.dna_address, &msg.agent_id);
                if self.trackdna_book.contains(&cell_id) {
                    self.log.e(&format!(
                        "({}) ##### DNA already tracked: {}",
                        self.name.clone(),
                        cell_id
                    ));
                    return Ok(());
                }
                self.trackdna_book.insert(cell_id);
                // Notify all Peers connected to this DNA of a new Peer connection.
                self.priv_send_all(
                    &msg.dna_address.clone(),
                    JsonProtocol::PeerConnected(PeerData {
                        agent_id: msg.agent_id.clone(),
                    })
                    .into(),
                )?;
                // Request all data lists from this agent
                self.priv_request_all_lists(&msg.dna_address, &msg.agent_id);
            }

            JsonProtocol::UntrackDna(msg) => {
                // Make sure we are already tracking this dna for this agent
                let cell_id = into_cell_id(&msg.dna_address, &msg.agent_id);
                if !self.trackdna_book.contains(&cell_id) {
                    self.log.w(&format!(
                        "Trying to untrack an already untracked DNA: {}",
                        cell_id
                    ));
                    return Ok(());
                }
                self.trackdna_book.remove(&cell_id);
            }

            JsonProtocol::SendMessage(msg) => {
                self.priv_serve_SendMessage(&msg)?;
            }
            JsonProtocol::HandleSendMessageResult(msg) => {
                self.priv_serve_HandleSendMessageResult(&msg)?;
            }
            JsonProtocol::FetchEntry(msg) => {
                self.priv_serve_FetchEntry(&msg)?;
            }
            JsonProtocol::HandleFetchEntryResult(msg) => {
                self.priv_serve_HandleFetchEntryResult(&msg)?;
            }

            JsonProtocol::PublishEntry(msg) => {
                self.priv_serve_PublishEntry(&msg)?;
            }

            JsonProtocol::FetchMeta(msg) => {
                self.priv_serve_FetchMeta(&msg)?;
            }
            JsonProtocol::HandleFetchMetaResult(msg) => {
                self.priv_serve_HandleFetchMetaResult(&msg)?;
            }

            JsonProtocol::PublishMeta(msg) => {
                self.priv_serve_PublishMeta(&msg)?;
            }

            // Our request for the publish_list has returned
            JsonProtocol::HandleGetPublishingEntryListResult(msg) => {
                self.priv_serve_HandleGetPublishingEntryListResult(&msg)?;
            }

            // Our request for the hold_list has returned
            JsonProtocol::HandleGetHoldingEntryListResult(msg) => {
                self.priv_serve_HandleGetHoldingEntryListResult(&msg);
            }

            // Our request for the publish_meta_list has returned
            JsonProtocol::HandleGetPublishingMetaListResult(msg) => {
                self.priv_serve_HandleGetPublishingMetaListResult(&msg)?;
            }

            // Our request for the hold_meta_list has returned
            JsonProtocol::HandleGetHoldingMetaListResult(msg) => {
                self.priv_serve_HandleGetHoldingMetaListResult(&msg);
            }

            _ => (),
        }
        Ok(())
    }
}

/// Private sends
impl InMemoryServer {
    /// Check if agent is tracking dna.
    /// If not, will try to send a FailureResult back to sender, if sender info is provided.
    /// Returns true if agent is tracking dna.
    fn priv_check_or_fail(
        &mut self,
        dna_address: &Address,
        agent_id: &str,
        maybe_sender_info: Option<(String, Option<String>)>,
    ) -> NetResult<bool> {
        let cell_id = into_cell_id(dna_address, agent_id);
        if self.trackdna_book.contains(&cell_id) {
            self.log.t(&format!(
                "---- '{}' check OK: {}",
                self.name.clone(),
                cell_id,
            ));
            return Ok(true);
        };
        if maybe_sender_info.is_none() {
            self.log.e(&format!(
                "#### '{}' check failed: {}",
                self.name.clone(),
                cell_id
            ));
            return Err(NetworkError::GenericError {
                error: "DNA not tracked by agent and no sender info.".to_string(),
            }
            .into());
        }
        let sender_info = maybe_sender_info.unwrap();
        let sender_agent_id = sender_info.0;
        let sender_request_id = sender_info.1.unwrap_or_default();
        let fail_msg = FailureResultData {
            dna_address: dna_address.clone(),
            request_id: sender_request_id,
            to_agent_id: sender_agent_id.clone(),
            error_info: json!(format!("DNA not tracked by agent")),
        };
        self.log.e(&format!(
            "#### '{}' check failed for {}.\n\t Sending failure {:?}",
            self.name.clone(),
            cell_id,
            fail_msg.clone()
        ));
        self.priv_send_one(
            dna_address,
            &sender_agent_id,
            JsonProtocol::FailureResult(fail_msg).into(),
        )?;
        Ok(false)
    }

    /// send a message to the appropriate channel based on dna_address::to_agent_id
    /// If cell_id is unknown, send back FailureResult to `maybe_sender_info`
    fn priv_send_one_with_cell_id(&mut self, cell_id: &str, data: Protocol) -> NetResult<()> {
        let maybe_sender = self.senders.get_mut(cell_id);
        if maybe_sender.is_none() {
            self.log.e(&format!(
                "#### ({}) error: No sender channel found for {}",
                self.name.clone(),
                cell_id,
            ));
            return Err(format_err!(
                "({}) No sender channel found for {}",
                self.name.clone(),
                cell_id,
            ));
        }
        let sender = maybe_sender.unwrap();
        self.log
            .d(&format!("<<<< '{}' send: {:?}", self.name.clone(), data));
        sender.send(data)?;
        Ok(())
    }
    /// send a message to the appropriate channel based on cell_id (dna_address::to_agent_id)
    /// If cell_id is unknown, send back FailureResult to `maybe_sender_info`
    fn priv_send_one(
        &mut self,
        dna_address: &Address,
        to_agent_id: &str,
        data: Protocol,
    ) -> NetResult<()> {
        let cell_id = into_cell_id(dna_address, to_agent_id);
        self.priv_send_one_with_cell_id(&cell_id, data)
    }

    /// send a message to all nodes connected with this dna address
    fn priv_send_all(&mut self, dna_address: &Address, data: Protocol) -> NetResult<()> {
        if let Some(arr) = self.senders_by_dna.get_mut(dna_address) {
            self.log.d(&format!(
                "<<<< '{}' send all: {:?} ({})",
                self.name.clone(),
                data.clone(),
                dna_address.clone()
            ));
            for (_k, val) in arr.iter_mut() {
                (*val).send(data.clone())?;
            }
        }
        Ok(())
    }
}

/// Private serve fns
impl InMemoryServer {
    /// we received a SendMessage message...
    /// normally this would travel over the network, then
    /// show up as a HandleSend message on the receiving agent
    /// Fabricate that message and deliver it to the receiving agent
    fn priv_serve_SendMessage(&mut self, msg: &MessageData) -> NetResult<()> {
        // Sender must be tracking
        let sender_info = Some((msg.from_agent_id.clone(), Some(msg.request_id.clone())));
        let is_tracking =
            self.priv_check_or_fail(&msg.dna_address, &msg.from_agent_id, sender_info.clone())?;
        if !is_tracking {
            return Ok(());
        }
        // Receiver must be tracking
        let is_tracking =
            self.priv_check_or_fail(&msg.dna_address, &msg.to_agent_id, sender_info)?;
        if !is_tracking {
            return Ok(());
        }
        // All good, relay message
        self.priv_send_one(
            &msg.dna_address,
            &msg.to_agent_id,
            JsonProtocol::HandleSendMessage(msg.clone()).into(),
        )?;
        // Done
        Ok(())
    }

    /// we received a HandleSendMessageResult message...
    /// normally this would travel over the network, then
    /// show up as a SendMessageResult message to the initial sender.
    /// Fabricate that message and deliver it to the initial sender.
    fn priv_serve_HandleSendMessageResult(&mut self, msg: &MessageData) -> NetResult<()> {
        // Sender must be tracking
        let sender_info = Some((msg.from_agent_id.clone(), Some(msg.request_id.clone())));
        let is_tracking =
            self.priv_check_or_fail(&msg.dna_address, &msg.from_agent_id, sender_info.clone())?;
        if !is_tracking {
            return Ok(());
        }
        // Receiver must be tracking
        let is_tracking =
            self.priv_check_or_fail(&msg.dna_address, &msg.to_agent_id, sender_info)?;
        if !is_tracking {
            return Ok(());
        }
        // All good, relay message
        self.priv_send_one(
            &msg.dna_address,
            &msg.to_agent_id,
            JsonProtocol::SendMessageResult(msg.clone()).into(),
        )?;
        Ok(())
    }

    // -- serve DHT Entry -- //

    /// on publish, we send store requests to all nodes connected on this dna
    fn priv_serve_PublishEntry(&mut self, msg: &EntryData) -> NetResult<()> {
        // Provider must be tracking
        let sender_info = Some((msg.provider_agent_id.clone(), None));
        let is_tracking =
            self.priv_check_or_fail(&msg.dna_address, &msg.provider_agent_id, sender_info)?;
        if !is_tracking {
            return Ok(());
        }
        // all good, book-keep publish
        bookkeep(
            &mut self.published_book,
            &msg.dna_address,
            &msg.provider_agent_id,
            &msg.entry_address,
            &msg.entry_address,
        );
        // #fulldht
        // have everyone store it (including self)
        self.priv_send_all(
            &msg.dna_address,
            JsonProtocol::HandleStoreEntry(msg.clone()).into(),
        )?;
        Ok(())
    }

    /// when someone makes a dht data request,
    /// this in-memory module routes it to the first node connected on that dna.
    /// this works because we send store requests to all connected nodes.
    /// If there is no other node for this DNA, send a FailureResult.
    fn priv_serve_FetchEntry(&mut self, msg: &FetchEntryData) -> NetResult<()> {
        // Provider must be tracking
        let sender_info = Some((msg.requester_agent_id.clone(), Some(msg.request_id.clone())));
        let is_tracking =
            self.priv_check_or_fail(&msg.dna_address, &msg.requester_agent_id, sender_info)?;
        if !is_tracking {
            return Ok(());
        }
        // #fulldht
        // Have the first known cell registered to that DNA respond
        match self.senders_by_dna.entry(msg.dna_address.to_owned()) {
            Entry::Occupied(mut e) => {
                if !e.get().is_empty() {
                    let (_k, r) = &e
                        .get_mut()
                        .iter()
                        .next()
                        .expect("No Cell is registered to track the DNA");
                    r.send(JsonProtocol::HandleFetchEntry(msg.clone()).into())?;
                    return Ok(());
                }
            }
            _ => unreachable!(),
        };

        // No node found, send an empty FetchEntryResultData
        // TODO: should send a FailureResult instead?
        let response = JsonProtocol::FetchEntryResult(FetchEntryResultData {
            request_id: msg.request_id.clone(),
            requester_agent_id: msg.requester_agent_id.clone(),
            dna_address: msg.dna_address.clone(),
            provider_agent_id: msg.requester_agent_id.clone(),
            entry_address: msg.entry_address.clone(),
            entry_content: json!(null),
        });
        self.priv_send_one(&msg.dna_address, &msg.requester_agent_id, response.into())?;
        // Done
        Ok(())
    }

    /// send back a response to a request for dht data
    fn priv_serve_HandleFetchEntryResult(&mut self, msg: &FetchEntryResultData) -> NetResult<()> {
        // Provider must be tracking
        let sender_info = Some((msg.provider_agent_id.clone(), Some(msg.request_id.clone())));
        let is_tracking = self.priv_check_or_fail(
            &msg.dna_address,
            &msg.provider_agent_id,
            sender_info.clone(),
        )?;
        if !is_tracking {
            return Ok(());
        }
        // Requester must be tracking
        let is_tracking = msg.requester_agent_id == ""
            || self.priv_check_or_fail(&msg.dna_address, &msg.requester_agent_id, sender_info)?;
        if !is_tracking {
            return Ok(());
        }
        // if its from our own request do a publish
        if self.request_book.contains_key(&msg.request_id) {
            let dht_data = EntryData {
                dna_address: msg.dna_address.clone(),
                provider_agent_id: msg.provider_agent_id.clone(),
                entry_address: msg.entry_address.clone(),
                entry_content: msg.entry_content.clone(),
            };
            self.priv_serve_PublishEntry(&dht_data)?;
            return Ok(());
        }
        // otherwise just send back to requester
        self.priv_send_one(
            &msg.dna_address,
            &msg.requester_agent_id,
            JsonProtocol::FetchEntryResult(msg.clone()).into(),
        )?;
        Ok(())
    }

    // -- serve DHT metadata -- //

    /// on publish, we send store requests to all nodes connected on this dna
    fn priv_serve_PublishMeta(&mut self, msg: &DhtMetaData) -> NetResult<()> {
        // Provider must be tracking
        let sender_info = Some((msg.provider_agent_id.clone(), None));
        let is_tracking =
            self.priv_check_or_fail(&msg.dna_address, &msg.provider_agent_id, sender_info)?;
        if !is_tracking {
            return Ok(());
        }
        // all good, book-keep every metaContent
        for content in msg.content_list.clone() {
            let meta_id = into_meta_id(&(
                msg.entry_address.clone(),
                msg.attribute.clone(),
                content.clone(),
            ));
            bookkeep(
                &mut self.published_book,
                &msg.dna_address,
                &msg.provider_agent_id,
                &msg.entry_address,
                &meta_id,
            );
        }
        // fully connected DHT so ask everyone to store the content.
        self.priv_send_all(
            &msg.dna_address,
            JsonProtocol::HandleStoreMeta(msg.clone()).into(),
        )
    }

    /// when someone makes a dht meta data request,
    /// this in-memory module routes it to the first node connected on that dna.
    /// this works because we also send store requests to all connected nodes.
    fn priv_serve_FetchMeta(&mut self, msg: &FetchMetaData) -> NetResult<()> {
        // Requester must be tracking
        let sender_info = Some((msg.requester_agent_id.clone(), Some(msg.request_id.clone())));
        let is_tracking =
            self.priv_check_or_fail(&msg.dna_address, &msg.requester_agent_id, sender_info)?;
        if !is_tracking {
            return Ok(());
        }
        // #fulldht
        // Have the first known cell registered to that DNA respond
        match self.senders_by_dna.entry(msg.dna_address.to_owned()) {
            Entry::Occupied(mut e) => {
                if !e.get().is_empty() {
                    let (_k, r) = &e
                        .get_mut()
                        .iter()
                        .next()
                        .expect("senders_by_dna.entry does not hold any value");
                    r.send(JsonProtocol::HandleFetchMeta(msg.clone()).into())?;
                    return Ok(());
                }
            }
            _ => unreachable!(),
        };
        // No node found, send an empty FetchMetaResultData
        // TODO: should send a FailureResult instead?
        let response = JsonProtocol::FetchMetaResult(FetchMetaResultData {
            request_id: msg.request_id.clone(),
            requester_agent_id: msg.requester_agent_id.clone(),
            dna_address: msg.dna_address.clone(),
            provider_agent_id: msg.requester_agent_id.clone(),
            entry_address: msg.entry_address.clone(),
            attribute: msg.attribute.clone(),
            content_list: vec![json!(null)],
        });
        self.priv_send_one(&msg.dna_address, &msg.requester_agent_id, response.into())?;
        // Done
        Ok(())
    }

    /// send back a response to a request for dht meta data
    fn priv_serve_HandleFetchMetaResult(&mut self, msg: &FetchMetaResultData) -> NetResult<()> {
        // Provider must be tracking
        let sender_info = Some((msg.provider_agent_id.clone(), Some(msg.request_id.clone())));
        let is_tracking = self.priv_check_or_fail(
            &msg.dna_address,
            &msg.provider_agent_id,
            sender_info.clone(),
        )?;
        if !is_tracking {
            return Ok(());
        }
        // Requester must be tracking
        let is_tracking = msg.requester_agent_id == ""
            || self.priv_check_or_fail(&msg.dna_address, &msg.requester_agent_id, sender_info)?;
        if !is_tracking {
            return Ok(());
        }
        // if its from our own request, resolve it
        if self.request_book.contains_key(&msg.request_id) {
            self.priv_resolve_HandleFetchMetaResult(msg)?;
            return Ok(());
        }
        // otherwise just send back to requester
        self.priv_send_one(
            &msg.dna_address,
            &msg.requester_agent_id,
            JsonProtocol::FetchMetaResult(msg.clone()).into(),
        )?;
        Ok(())
    }

    /// Resolve our own HandleFetchMeta request:
    ///   - do a publish for each new/unknown meta content
    fn priv_resolve_HandleFetchMetaResult(&mut self, msg: &FetchMetaResultData) -> NetResult<()> {
        let cell_id = into_cell_id(&msg.dna_address, &msg.provider_agent_id);
        for meta_content in msg.content_list.clone() {
            let meta_id = into_meta_id(&(
                msg.entry_address.clone(),
                msg.attribute.clone(),
                meta_content.clone(),
            ));
            if book_has(
                &self.published_book,
                cell_id.clone(),
                &msg.entry_address,
                &meta_id,
            ) {
                continue;
            }
            self.log.t(&format!(
                "Publishing missing Meta: {}",
                meta_content.clone()
            ));
            let meta_data = DhtMetaData {
                dna_address: msg.dna_address.clone(),
                provider_agent_id: msg.provider_agent_id.clone(),
                entry_address: msg.entry_address.clone(),
                attribute: msg.attribute.clone(),
                content_list: vec![meta_content],
            };
            self.priv_serve_PublishMeta(&meta_data)?;
        }
        Ok(())
    }

    /// Received response from our request for the 'publish_list'
    /// For each data not already published, request it in order to publish it ourselves.
    fn priv_serve_HandleGetPublishingEntryListResult(
        &mut self,
        msg: &EntryListData,
    ) -> NetResult<()> {
        let cell_id = self
            .priv_check_request(&msg.request_id)
            .expect("Not our request")
            .to_string();
        self.log.d(&format!(
            "---- HandleGetPublishingDataListResult: cell_id = '{}'",
            cell_id,
        ));
        // Compare with already published list
        // For each data not already published, request it and publish it ourselves.
        for entry_address in msg.entry_address_list.clone() {
            if book_has_entry(&self.published_book, cell_id.clone(), &entry_address) {
                continue;
            }
            let request_id = self.priv_create_request_with_cell_id(&cell_id);
            self.priv_send_one_with_cell_id(
                &cell_id,
                JsonProtocol::HandleFetchEntry(FetchEntryData {
                    requester_agent_id: String::new(),
                    request_id,
                    dna_address: msg.dna_address.clone(),
                    entry_address,
                })
                .into(),
            )?;
        }
        Ok(())
    }

    /// Received response from our request for the 'holding_list'
    fn priv_serve_HandleGetHoldingEntryListResult(&mut self, msg: &EntryListData) {
        let cell_id = self
            .priv_check_request(&msg.request_id)
            .expect("Not our request")
            .to_string();
        self.log.d(&format!(
            "---- HandleGetHoldingEntryListResult: cell_id = '{}'",
            cell_id,
        ));
        // Compare with current stored_book
        // For each data not already holding, add it to stored_data_book?
        for entry_address in msg.entry_address_list.clone() {
            if book_has_entry(&self.stored_book, cell_id.clone(), &entry_address) {
                continue;
            }
            bookkeep_with_cell_id(
                &mut self.stored_book,
                cell_id.clone(),
                &entry_address,
                &entry_address,
            );
        }
    }

    /// Received response from our request for the 'publish_list'
    /// For each data not already published, request it in order to publish it ourselves.
    fn priv_serve_HandleGetPublishingMetaListResult(
        &mut self,
        msg: &MetaListData,
    ) -> NetResult<()> {
        let cell_id = self
            .priv_check_request(&msg.request_id)
            .expect("Not our request")
            .to_string();
        self.log.d(&format!(
            "---- HandleGetPublishingMetaListResult: cell_id = '{}'",
            cell_id,
        ));
        // Compare with already published list
        // For each metadata not already published, request it and publish it ourselves.
        let mut requested_meta_key = Vec::new();
        for meta_tuple in msg.meta_list.clone() {
            let meta_id = into_meta_id(&meta_tuple);
            // dont send request for a known meta
            if book_has(
                &self.published_book,
                cell_id.clone(),
                &meta_tuple.0,
                &meta_id,
            ) {
                continue;
            }
            // dont send same request twice
            let meta_key = (meta_tuple.0.clone(), meta_tuple.1.clone());
            if requested_meta_key.contains(&meta_key) {
                continue;
            }
            requested_meta_key.push(meta_key);
            // send request for that meta_key
            let request_id = self.priv_create_request_with_cell_id(&cell_id);
            let fetch_meta = FetchMetaData {
                requester_agent_id: String::new(),
                request_id,
                dna_address: msg.dna_address.clone(),
                entry_address: meta_tuple.0,
                attribute: meta_tuple.1,
            };
            self.priv_send_one_with_cell_id(
                &cell_id,
                JsonProtocol::HandleFetchMeta(fetch_meta).into(),
            )?;
        }
        Ok(())
    }

    /// Received response from our request for the 'holding_meta_list'
    fn priv_serve_HandleGetHoldingMetaListResult(&mut self, msg: &MetaListData) {
        let cell_id = self
            .priv_check_request(&msg.request_id)
            .expect("Not our request")
            .to_string();
        self.log.d(&format!(
            "---- HandleGetHoldingMetaListResult: cell_id = '{}'",
            cell_id,
        ));
        // Compare with current stored_meta_book
        // For each data not already holding, add it to stored_meta_book?
        for meta_tuple in msg.meta_list.clone() {
            let meta_id = into_meta_id(&meta_tuple);
            if book_has(&self.stored_book, cell_id.clone(), &meta_tuple.0, &meta_id) {
                continue;
            }
            bookkeep_with_cell_id(
                &mut self.stored_book,
                cell_id.clone(),
                &meta_tuple.0,
                &meta_id,
            );
        }
    }
}
