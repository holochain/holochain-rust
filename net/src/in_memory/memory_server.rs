//! provides in-memory p2p "server" for use in scenario testing
//! the server connects all the memory_workers together, so there is no real gossiping going around.
//! Could have pluggable DHT strategy. Full-sync currently hard-coded: #fullsync

#![allow(non_snake_case)]

use super::memory_book::*;
use crate::{
    connection::{
        json_protocol::{
            EntryListData, FetchEntryData, FetchEntryResultData, GenericResultData, GetListData,
            JsonProtocol, MessageData, PeerData, ProvidedEntryData, QueryEntryData,
            QueryEntryResultData, StoreEntryAspectData,
        },
        protocol::Protocol,
        NetResult,
    },
    error::NetworkError,
    tweetlog::*,
};
use holochain_persistence_api::cas::content::Address;
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
    // keep track of senders by ChainId (dna_address::agent_id)
    senders: HashMap<ChainId, mpsc::Sender<Protocol>>,
    // keep track of agents by dna_address
    senders_by_dna: HashMap<Address, HashMap<Address, mpsc::Sender<Protocol>>>,
    // Unique identifier
    name: String,
    // Keep track of connected clients
    client_count: usize,

    // All published data book: chain_id -> entry_addresses
    authored_book: ChainBook,
    // All stored data book: chain_id -> entry_addresses
    stored_book: ChainBook,

    // Keep track of which DNAs are "tracked"
    trackdna_book: HashSet<ChainId>,
    // Keep track of requests authored by the server
    // request_id -> chain_id
    request_book: HashMap<RequestId, ChainId>,
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
    fn priv_create_request(&mut self, dna_address: &Address, agent_id: &Address) -> RequestId {
        let chain_id = into_chain_id(dna_address, agent_id);
        self.priv_create_request_with_chain_id(&chain_id)
    }

    /// generate a new request, return the request_id
    fn priv_create_request_with_chain_id(&mut self, chain_id: &ChainId) -> RequestId {
        let req_id = self.priv_generate_request_id();
        self.request_book
            .insert(req_id.clone(), chain_id.to_string());
        req_id
    }

    /// Check if its our own request and return ChainId
    fn priv_check_request(&self, request_id: &RequestId) -> Option<&ChainId> {
        self.log.t(&format!(
            "---- priv_check_request('{}') in {:?} ?",
            request_id,
            self.request_book.clone(),
        ));
        self.request_book.get(&request_id.clone())
    }

    /// Send all Get*Lists requests to agent
    fn priv_request_all_lists(&mut self, dna_address: &Address, agent_id: &Address) {
        // Entry
        // Request this agent's published entries
        let request_id = self.priv_create_request(dna_address, agent_id);
        self.priv_send_one(
            dna_address,
            agent_id,
            JsonProtocol::HandleGetAuthoringEntryList(GetListData {
                request_id,
                provider_agent_id: agent_id.clone(),
                dna_address: dna_address.clone(),
            })
            .into(),
        )
        .expect("Sending HandleGetAuthoringEntryList failed");
        // Request this agent's holding entries
        let request_id = self.priv_create_request(dna_address, agent_id);
        self.priv_send_one(
            dna_address,
            agent_id,
            JsonProtocol::HandleGetGossipingEntryList(GetListData {
                request_id,
                provider_agent_id: agent_id.clone(),
                dna_address: dna_address.clone(),
            })
            .into(),
        )
        .expect("Sending HandleGetHoldingEntryList failed");
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
            authored_book: HashMap::new(),
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

    /// register a chain's handler with the server (for message routing)
    pub fn register_chain(
        &mut self,
        dna_address: &Address,
        agent_id: &Address,
        sender: mpsc::Sender<Protocol>,
    ) -> NetResult<()> {
        self.senders
            .insert(into_chain_id(dna_address, agent_id), sender.clone());
        match self.senders_by_dna.entry(dna_address.to_owned()) {
            Entry::Occupied(mut e) => {
                e.get_mut().insert(agent_id.clone(), sender.clone());
            }
            Entry::Vacant(e) => {
                let mut map = HashMap::new();
                map.insert(agent_id.clone(), sender.clone());
                e.insert(map);
            }
        };
        Ok(())
    }

    /// unregister a chain's handler with the server (for message routing)
    pub fn unregister_chain(&mut self, dna_address: &Address, agent_id: &Address) {
        let chain_id = into_chain_id(dna_address, agent_id);
        self.log.d(&format!("unregistering '{}'", chain_id));
        let maybe_sender = self.senders.remove(&chain_id);
        if maybe_sender.is_none() {
            return;
        }
        match self.senders_by_dna.entry(dna_address.to_owned()) {
            Entry::Occupied(mut senders) => {
                senders.get_mut().remove(agent_id);
            }
            Entry::Vacant(_) => unreachable!(),
        };
        self.log.d(&format!("unregistering '{}' DONE", chain_id));
    }

    /// process a message sent by a node to the "network"
    pub fn serve(&mut self, data: Protocol) -> NetResult<()> {
        self.log
            .d(&format!(">>>> '{}' recv: {:?}", self.name.clone(), data));
        // serve only JsonProtocol
        let maybe_json_msg = JsonProtocol::try_from(&data);
        if maybe_json_msg.is_err() {
            return Ok(());
        };
        // Note: use same order as the enum
        match maybe_json_msg.as_ref().unwrap() {
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
                    let maybe_chain_id = self.priv_check_request(&msg.request_id);
                    if maybe_chain_id.is_some() {
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
                let chain_id = into_chain_id(&msg.dna_address, &msg.agent_id);
                if self.trackdna_book.contains(&chain_id) {
                    self.log.e(&format!(
                        "({}) ##### DNA already tracked: {}",
                        self.name.clone(),
                        chain_id
                    ));
                    return Ok(());
                }
                self.trackdna_book.insert(chain_id);
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
                let chain_id = into_chain_id(&msg.dna_address, &msg.agent_id);
                if !self.trackdna_book.contains(&chain_id) {
                    self.log.w(&format!(
                        "Trying to untrack an already untracked DNA: {}",
                        chain_id
                    ));
                    return Ok(());
                }
                self.trackdna_book.remove(&chain_id);
            }

            JsonProtocol::SendMessage(msg) => {
                self.priv_serve_SendMessage(&msg)?;
            }
            JsonProtocol::HandleSendMessageResult(msg) => {
                self.priv_serve_HandleSendMessageResult(&msg)?;
            }

            JsonProtocol::PublishEntry(msg) => {
                self.priv_serve_PublishEntry(&msg)?;
            }
            JsonProtocol::HandleFetchEntryResult(msg) => {
                self.priv_serve_HandleFetchEntryResult(&msg)?;
            }
            JsonProtocol::QueryEntry(msg) => {
                self.priv_serve_QueryEntry(&msg)?;
            }
            JsonProtocol::HandleQueryEntryResult(msg) => {
                self.priv_serve_HandleQueryEntryResult(&msg)?;
            }

            // Our request for the publish_list has returned
            JsonProtocol::HandleGetAuthoringEntryListResult(msg) => {
                self.priv_serve_HandleGetAuthoringEntryListResult(&msg)?;
            }

            // Our request for the hold_list has returned
            JsonProtocol::HandleGetGossipingEntryListResult(msg) => {
                self.priv_serve_HandleGetGossipingEntryListResult(&msg);
            }

            _ => {
                self.log.w(&format!("unexpected {:?}", &maybe_json_msg));
            }
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
        agent_id: &Address,
        maybe_sender_info: Option<(Address, Option<String>)>,
    ) -> NetResult<bool> {
        let chain_id = into_chain_id(dna_address, agent_id);
        if self.trackdna_book.contains(&chain_id) {
            self.log.t(&format!(
                "---- '{}' check OK: {}",
                self.name.clone(),
                chain_id,
            ));
            return Ok(true);
        };
        if maybe_sender_info.is_none() {
            self.log.e(&format!(
                "#### '{}' check failed: {}",
                self.name.clone(),
                chain_id
            ));
            return Err(NetworkError::GenericError {
                error: "DNA not tracked by agent and no sender info.".to_string(),
            }
            .into());
        }
        let sender_info = maybe_sender_info.unwrap();
        let sender_agent_id = sender_info.0;
        let sender_request_id = sender_info.1.unwrap_or_default();
        let fail_msg = GenericResultData {
            dna_address: dna_address.clone(),
            request_id: sender_request_id,
            to_agent_id: sender_agent_id.clone(),
            result_info: "DNA not tracked by agent".into(),
        };
        self.log.e(&format!(
            "#### '{}' check failed for {}.\n\t Sending failure {:?}",
            self.name.clone(),
            chain_id,
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
    /// If chain_id is unknown, send back FailureResult to `maybe_sender_info`
    fn priv_send_one_with_chain_id(&mut self, chain_id: &str, data: Protocol) -> NetResult<()> {
        let maybe_sender = self.senders.get_mut(chain_id);
        if maybe_sender.is_none() {
            self.log.e(&format!(
                "#### ({}) error: No sender channel found for {}",
                self.name.clone(),
                chain_id,
            ));
            return Err(format_err!(
                "({}) No sender channel found for {}",
                self.name.clone(),
                chain_id,
            ));
        }
        let sender = maybe_sender.unwrap();
        self.log
            .d(&format!("<<<< '{}' send: {:?}", self.name.clone(), data));
        sender.send(data)?;
        Ok(())
    }
    /// send a message to the appropriate channel based on chain_id (dna_address::to_agent_id)
    /// If chain_id is unknown, send back FailureResult to `maybe_sender_info`
    fn priv_send_one(
        &mut self,
        dna_address: &Address,
        to_agent_id: &Address,
        data: Protocol,
    ) -> NetResult<()> {
        let chain_id = into_chain_id(dna_address, to_agent_id);
        self.priv_send_one_with_chain_id(&chain_id, data)
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

    // -- serve Publish Entry -- //

    /// on publish, we send store requests to all nodes connected on this dna
    fn priv_serve_PublishEntry(&mut self, msg: &ProvidedEntryData) -> NetResult<()> {
        // Provider must be tracking
        let sender_info = Some((msg.provider_agent_id.clone(), None));
        let is_tracking = self.priv_check_or_fail(
            &msg.dna_address,
            &msg.provider_agent_id.clone(),
            sender_info,
        )?;
        if !is_tracking {
            return Ok(());
        }

        // Store every aspect
        for aspect in msg.entry.aspect_list.clone() {
            let chain_id = into_chain_id(&msg.dna_address, &msg.provider_agent_id);
            // Publish is authoring unless its broadcasting an aspect we are storing
            if !book_has_aspect(
                &self.stored_book,
                chain_id,
                &msg.entry.entry_address,
                &aspect.aspect_address,
            ) {
                bookkeep(
                    &mut self.authored_book,
                    &msg.dna_address,
                    &msg.provider_agent_id,
                    &msg.entry.entry_address,
                    &aspect.aspect_address,
                );
            }
            let store_msg = StoreEntryAspectData {
                request_id: self.priv_generate_request_id(),
                dna_address: msg.dna_address.clone(),
                provider_agent_id: msg.provider_agent_id.clone(),
                entry_address: msg.entry.entry_address.clone(),
                entry_aspect: aspect,
            };
            // #fullsync
            // Broadcast: have everyone store it (including self)
            self.priv_send_all(
                &msg.dna_address,
                JsonProtocol::HandleStoreEntryAspect(store_msg).into(),
            )?;
        }
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
        //        // Requester must be tracking
        //        let is_tracking = msg.requester_agent_id == ""
        //            || self.priv_check_or_fail(&msg.dna_address, &msg.requester_agent_id, sender_info)?;
        //        if !is_tracking {
        //            return Ok(());
        //        }

        // Should be from our own request do a publish
        if !self.request_book.contains_key(&msg.request_id) {
            // FIXME return Err instead
            return Ok(());
        }
        let dht_data = ProvidedEntryData {
            dna_address: msg.dna_address.clone(),
            provider_agent_id: msg.provider_agent_id.clone(),
            entry: msg.entry.clone(),
        };
        self.priv_serve_PublishEntry(&dht_data)?;
        Ok(())
    }

    // -- serve QueryEntry -- //

    fn priv_serve_QueryEntry(&mut self, msg: &QueryEntryData) -> NetResult<()> {
        // Provider must be tracking
        let sender_info = Some((msg.requester_agent_id.clone(), Some(msg.request_id.clone())));
        let is_tracking =
            self.priv_check_or_fail(&msg.dna_address, &msg.requester_agent_id, sender_info)?;
        if !is_tracking {
            return Ok(());
        }
        // #fullsync
        // Have the first known chain registered to that DNA respond
        match self.senders_by_dna.entry(msg.dna_address.to_owned()) {
            Entry::Occupied(mut e) => {
                if !e.get().is_empty() {
                    let (_k, r) = &e
                        .get_mut()
                        .iter()
                        .next()
                        .expect("No Chain is registered to track the DNA");
                    r.send(JsonProtocol::HandleQueryEntry(msg.clone()).into())?;
                    return Ok(());
                }
            }
            _ => unreachable!(),
        };

        // No node found, send an empty FetchEntryResultData
        // TODO: should send a FailureResult instead?
        let response = JsonProtocol::QueryEntryResult(QueryEntryResultData {
            dna_address: msg.dna_address.clone(),
            entry_address: msg.entry_address.clone(),
            request_id: msg.request_id.clone(),
            requester_agent_id: msg.requester_agent_id.clone(),
            responder_agent_id: msg.requester_agent_id.clone(),
            query_result: vec![],
        });
        self.priv_send_one(&msg.dna_address, &msg.requester_agent_id, response.into())?;
        // Done
        Ok(())
    }

    fn priv_serve_HandleQueryEntryResult(&mut self, msg: &QueryEntryResultData) -> NetResult<()> {
        // Provider/Responder must be tracking
        let sender_info = Some((msg.responder_agent_id.clone(), Some(msg.request_id.clone())));
        let is_tracking = self.priv_check_or_fail(
            &msg.dna_address,
            &msg.responder_agent_id.clone(),
            sender_info.clone(),
        )?;
        if !is_tracking {
            return Ok(());
        }
        // Requester must be tracking
        let is_tracking = msg.requester_agent_id.to_string() == ""
            || self.priv_check_or_fail(&msg.dna_address, &msg.requester_agent_id, sender_info)?;
        if !is_tracking {
            return Ok(());
        }
        // otherwise just send back to requester
        self.priv_send_one(
            &msg.dna_address,
            &msg.requester_agent_id,
            JsonProtocol::QueryEntryResult(msg.clone()).into(),
        )?;
        Ok(())
    }

    // -- serve Get List -- //

    /// Received response from our request for the 'publish_list'
    /// For each data not already published, request it in order to publish it ourselves.
    fn priv_serve_HandleGetAuthoringEntryListResult(
        &mut self,
        msg: &EntryListData,
    ) -> NetResult<()> {
        let chain_id = self
            .priv_check_request(&msg.request_id)
            .expect("Not our request")
            .to_string();
        self.log.d(&format!(
            "---- HandleGetAuthoringEntryListResult: chain_id = '{}'",
            chain_id,
        ));
        // Compare with already authored list
        // For each aspect not already authored, fetch it and publish it ourselves.
        for (entry_address, aspect_address_list) in msg.address_map.clone() {
            for aspect_address in aspect_address_list {
                if book_has_aspect(
                    &self.authored_book,
                    chain_id.clone(),
                    &entry_address,
                    &aspect_address,
                ) {
                    continue;
                }
                let request_id = self.priv_create_request_with_chain_id(&chain_id);
                self.priv_send_one_with_chain_id(
                    &chain_id,
                    JsonProtocol::HandleFetchEntry(FetchEntryData {
                        dna_address: msg.dna_address.clone(),
                        provider_agent_id: undo_chain_id(&chain_id).1,
                        request_id,
                        entry_address: entry_address.clone(),
                        aspect_address_list: Some(vec![aspect_address]),
                    })
                    .into(),
                )?;
            }
        }
        Ok(())
    }

    /// Received response from our request for the 'holding_list'
    fn priv_serve_HandleGetGossipingEntryListResult(&mut self, msg: &EntryListData) {
        let chain_id = self
            .priv_check_request(&msg.request_id)
            .expect("Not our request")
            .to_string();
        self.log.d(&format!(
            "---- HandleGetHoldingEntryListResult: chain_id = '{}'",
            chain_id,
        ));
        // Compare with current stored_book
        // For each data not already holding, add it to stored_data_book?
        for (entry_address, aspect_address_list) in msg.address_map.clone() {
            for aspect_address in aspect_address_list {
                if book_has_aspect(
                    &self.stored_book,
                    chain_id.clone(),
                    &entry_address,
                    &aspect_address,
                ) {
                    continue;
                }
                bookkeep_with_chain_id(
                    &mut self.stored_book,
                    chain_id.clone(),
                    &entry_address,
                    &aspect_address,
                );
                // Ask for the new aspect since in-memory mode doesnt gossip
                let request_id = self.priv_create_request_with_chain_id(&chain_id);
                let _ = self.priv_send_one_with_chain_id(
                    &chain_id,
                    JsonProtocol::HandleFetchEntry(FetchEntryData {
                        dna_address: msg.dna_address.clone(),
                        provider_agent_id: undo_chain_id(&chain_id).1,
                        request_id,
                        entry_address: entry_address.clone(),
                        aspect_address_list: Some(vec![aspect_address]),
                    })
                    .into(),
                );
            }
        }
    }
}
