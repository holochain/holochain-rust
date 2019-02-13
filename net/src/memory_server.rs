//! provides fake in-memory p2p worker for use in scenario testing

#![allow(non_snake_case)]

use crate::{error::NetworkError, tweetlog::*};
use holochain_core_types::{cas::content::Address, entry::EntryWithMeta, hash::HashString};
use holochain_net_connection::{
    json_protocol::{
        DhtMetaData, EntryData, EntryListData, FailureResultData, FetchEntryData,
        FetchEntryResultData, FetchMetaData, FetchMetaResultData, GetListData, JsonProtocol,
        MessageData, MetaListData, MetaTuple, PeerData,
    },
    protocol::Protocol,
    NetResult,
};
use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    convert::TryFrom,
    sync::{mpsc, Mutex, RwLock},
};

type BucketId = String;
type RequestId = String;

/// Type for holding list of addresses per dna+agent_id
/// i.e. map of bucket_id -> addresses
type AddressBook = HashMap<BucketId, Vec<Address>>;

/// Type for holding a map of 'network_name -> InMemoryServer'
type InMemoryServerMap = HashMap<String, Mutex<InMemoryServer>>;

/// this is the actual memory space for our in-memory servers
lazy_static! {
    pub(crate) static ref MEMORY_SERVER_MAP: RwLock<InMemoryServerMap> =
        RwLock::new(HashMap::new());
}

/// return a BucketId out of a dna_address and agent_id
fn into_bucket_id(dna_address: &Address, agent_id: &str) -> BucketId {
    format!("{}::{}", dna_address, agent_id)
}

/// return a unique identifier out of a entry_address and attribute
pub fn into_meta_id(meta_tuple: &MetaTuple) -> Address {
    HashString::from(format!(
        "{}||{}||{}",
        meta_tuple.0, meta_tuple.1, meta_tuple.2
    ))
}

/// Add an address to a book
fn bookkeep_address_with_bucket(book: &mut AddressBook, bucket_id: BucketId, address: &Address) {
    // Append to existing address list if there is one
    {
        let maybe_vec_address = book.get_mut(&bucket_id);
        if let Some(vec_address) = maybe_vec_address {
            vec_address.push(address.clone());
            return;
        }
    } // unborrow book
      // None: Create and add a new address list
    let vec = vec![address.clone()];
    book.insert(bucket_id, vec);
}

/// Add an address to a book (sugar)
fn bookkeep_address(
    book: &mut AddressBook,
    dna_address: &Address,
    agent_id: &str,
    address: &Address,
) {
    let bucket_id = into_bucket_id(dna_address, agent_id);
    bookkeep_address_with_bucket(book, bucket_id, address);
}

/// Remove an address from a book
/// Return true if address exists and has been successfully removed.
fn _unbookkeep_address(
    book: &mut AddressBook,
    dna_address: &Address,
    agent_id: &str,
    address: &Address,
) -> bool {
    let bucket_id = into_bucket_id(dna_address, agent_id);
    let maybe_vec_address = book.get_mut(&bucket_id);
    if let Some(vec_address) = maybe_vec_address {
        let result = vec_address.remove_item(address);
        return result.is_some();
    }
    false
}

/// a global server for routing messages between agents in-memory
pub(crate) struct InMemoryServer {
    // keep track of senders by `dna_address::agent_id`
    senders: HashMap<BucketId, mpsc::Sender<Protocol>>,
    // keep track of agents by dna_address
    senders_by_dna: HashMap<Address, HashMap<String, mpsc::Sender<Protocol>>>,
    // Unique identifier
    name: String,
    // Keep track of connected clients
    client_count: usize,

    // published data book: bucket_id -> entry_addresses
    published_entry_book: AddressBook,
    // stored data book: bucket_id -> entry_addresses
    stored_entry_book: AddressBook,
    // published meta data book: bucket_id -> MetaIds
    published_meta_book: AddressBook,
    // stored meta data book: bucket_id -> MetaIds
    stored_meta_book: AddressBook,

    // Keep track of which DNAs are tracked... String should be BucketId
    trackdna_book: HashSet<BucketId>,
    // request book: request_id -> bucket_id
    request_book: HashMap<RequestId, BucketId>,
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
        let bucket_id = into_bucket_id(dna_address, agent_id);
        self.priv_create_request_with_bucket(&bucket_id)
    }

    /// generate a new request, return the request_id
    fn priv_create_request_with_bucket(&mut self, bucket_id: &BucketId) -> RequestId {
        let req_id = self.priv_generate_request_id();
        self.request_book
            .insert(req_id.clone(), bucket_id.to_string());
        req_id
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
            published_entry_book: HashMap::new(),
            stored_entry_book: HashMap::new(),
            published_meta_book: HashMap::new(),
            stored_meta_book: HashMap::new(),
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

    /// register a data handler with the server (for message routing)
    pub fn register(
        &mut self,
        dna_address: &Address,
        agent_id: &str,
        sender: mpsc::Sender<Protocol>,
    ) -> NetResult<()> {
        self.senders
            .insert(into_bucket_id(dna_address, agent_id), sender.clone());
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

    /// unregister a data handler with the server (for message routing)
    pub fn unregister(&mut self, dna_address: &Address, agent_id: &str) {
        let bucket_id = into_bucket_id(dna_address, agent_id);
        self.log.d(&format!("unregistering '{}'", bucket_id));
        let maybe_sender = self.senders.remove(&bucket_id);
        if maybe_sender.is_none() {
            return;
        }
        // let sender = maybe_sender.unwrap();
        match self.senders_by_dna.entry(dna_address.to_owned()) {
            Entry::Occupied(mut senders) => {
                senders.get_mut().remove(agent_id.clone());
            }
            Entry::Vacant(_) => (),
        };
        self.log.d(&format!("unregistering '{}' DONE", bucket_id));
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
                let maybe_bucket_id = self.priv_check_request(&msg.request_id);
                if let Some(_) = maybe_bucket_id {
                    self.log.d(&format!(
                        "---- '{}' internal request failed: {:?}",
                        self.name.clone(),
                        msg.clone(),
                    ));
                    return Ok(());
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
                let bucket_id = into_bucket_id(&msg.dna_address, &msg.agent_id);
                if self.trackdna_book.contains(&bucket_id) {
                    self.log.e(&format!(
                        "({}) ##### DNA already tracked: {}",
                        self.name.clone(),
                        bucket_id
                    ));
                    return Ok(());
                }
                self.trackdna_book.insert(bucket_id);
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
                let bucket_id = into_bucket_id(&msg.dna_address, &msg.agent_id);
                if !self.trackdna_book.contains(&bucket_id) {
                    self.log.w(&format!(
                        "Trying to untrack an already untracked DNA: {}",
                        bucket_id
                    ));
                    return Ok(());
                }
                self.trackdna_book.remove(&bucket_id);
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
        let bucket_id = into_bucket_id(dna_address, agent_id);
        //        self.log.d(&format!(
        //            "---- '{}' checking '{}' ...",
        //            self.name.clone(),
        //            bucket_id,
        //        ));
        if self.trackdna_book.contains(&bucket_id) {
            self.log.t(&format!(
                "---- '{}' check OK: {}",
                self.name.clone(),
                bucket_id,
            ));
            return Ok(true);
        };
        if maybe_sender_info.is_none() {
            self.log.e(&format!(
                "#### '{}' check failed: {}",
                self.name.clone(),
                bucket_id
            ));
            return Err(NetworkError::GenericError {
                error: "DNA not tracked by agent and no sender info.".to_string(),
            }
            .into());
        }
        let sender_info = maybe_sender_info.unwrap();
        let sender_agent_id = sender_info.0;
        let sender_request_id = match sender_info.1 {
            None => String::new(),
            Some(req_id) => req_id,
        };
        let fail_msg = FailureResultData {
            dna_address: dna_address.clone(),
            request_id: sender_request_id,
            to_agent_id: sender_agent_id.clone(),
            error_info: json!(format!("DNA not tracked by agent")),
        };
        self.log.e(&format!(
            "#### '{}' check failed for {}.\n Sending failure {:?}",
            self.name.clone(),
            bucket_id,
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
    /// If bucketId is unknown, send back FailureResult to `maybe_sender_info`
    fn priv_send_one_with_bucket(&mut self, bucket_id: &str, data: Protocol) -> NetResult<()> {
        let maybe_sender = self.senders.get_mut(bucket_id);
        if maybe_sender.is_none() {
            self.log.e(&format!(
                "#### ({}) error: No sender channel found for {}",
                self.name.clone(),
                bucket_id,
            ));
            return Err(format_err!(
                "({}) No sender channel found for {}",
                self.name.clone(),
                bucket_id,
            ));
        }
        let sender = maybe_sender.unwrap();
        self.log
            .d(&format!("<<<< '{}' send: {:?}", self.name.clone(), data));
        sender.send(data)?;
        Ok(())
    }
    /// send a message to the appropriate channel based on bucketId (dna_address::to_agent_id)
    /// If bucketId is unknown, send back FailureResult to `maybe_sender_info`
    fn priv_send_one(
        &mut self,
        dna_address: &Address,
        to_agent_id: &str,
        data: Protocol,
    ) -> NetResult<()> {
        let bucked_id = into_bucket_id(dna_address, to_agent_id);
        self.priv_send_one_with_bucket(&bucked_id, data)
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
        bookkeep_address(
            &mut self.published_entry_book,
            &msg.dna_address,
            &msg.provider_agent_id,
            &msg.entry_address,
        );
        // fully connected DHT, so have everyone store it.
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
        // Find other node and forward request
        match self.senders_by_dna.entry(msg.dna_address.to_owned()) {
            Entry::Occupied(e) => {
                for (agent_id, sender) in e.get() {
                    let bucket_id = into_bucket_id(&msg.dna_address, agent_id);
                    let published = self.published_entry_book.get(&bucket_id);
                    let stored = self.stored_entry_book.get(&bucket_id);
                    let has_entry = (published.is_some()
                        && published.unwrap().contains(&msg.entry_address))
                        || (stored.is_some() && stored.unwrap().contains(&msg.entry_address));
                    if has_entry {
                        self.log.d(&format!(
                            "<<<< '{}' sending to ({}): {:?}",
                            self.name.clone(),
                            agent_id,
                            msg.clone()
                        ));
                        let msg: Protocol = JsonProtocol::HandleFetchEntry(msg.clone()).into();
                        sender.send(msg)?;
                        return Ok(());
                    }
                }
            }
            _ => (),
        };
        // no other node found, send a FailureResult.
        //        let response = JsonProtocol::FailureResult(FailureResultData {
        //            request_id: msg.request_id.clone(),
        //            dna_address: msg.dna_address.clone(),
        //            to_agent_id: msg.requester_agent_id.clone(),
        //            error_info: json!("could not find an agent holding this Entry"),
        //        });
        let maybe_entry: Option<EntryWithMeta> = None;
        let response = JsonProtocol::HandleFetchEntryResult(FetchEntryResultData {
            request_id: msg.request_id.clone(),
            requester_agent_id: msg.requester_agent_id.clone(),
            dna_address: msg.dna_address.clone(),
            provider_agent_id: msg.requester_agent_id.clone(),
            entry_address: msg.entry_address.clone(),
            entry_content: serde_json::from_str(&serde_json::to_string(&maybe_entry).unwrap())
                .unwrap(),
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
            bookkeep_address(
                &mut self.published_meta_book,
                &msg.dna_address,
                &msg.provider_agent_id,
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
        // Relay fetchMeta to first agent registered to that dna.
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
            _ => (),
        };
        // no other node found, send a FailureResult.
        self.priv_send_one(
            &msg.dna_address,
            &msg.requester_agent_id,
            JsonProtocol::FailureResult(FailureResultData {
                request_id: msg.request_id.clone(),
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
        // if its from our own request, do a publish for each new/unknown meta content
        if self.request_book.contains_key(&msg.request_id) {
            let bucket_id = into_bucket_id(&msg.dna_address, &msg.provider_agent_id);
            let known_published_meta_list = match self.published_meta_book.get(&bucket_id) {
                Some(list) => list.clone(),
                None => Vec::new(),
            };
            for content in msg.content_list.clone() {
                let meta_id = into_meta_id(&(
                    msg.entry_address.clone(),
                    msg.attribute.clone(),
                    content.clone(),
                ));
                if known_published_meta_list.contains(&meta_id) {
                    continue;
                }
                self.log.t(&format!("Asking for Meta: {}", content.clone()));
                let meta_data = DhtMetaData {
                    dna_address: msg.dna_address.clone(),
                    provider_agent_id: msg.provider_agent_id.clone(),
                    entry_address: msg.entry_address.clone(),
                    attribute: msg.attribute.clone(),
                    content_list: vec![content],
                };
                self.priv_serve_PublishMeta(&meta_data)?;
            }
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

    ///
    fn priv_check_request(&mut self, request_id: &RequestId) -> Option<BucketId> {
        // Get bucket_id and make sure its our request
        let bucket_id;
        {
            self.log.t(&format!(
                "---- priv_check_request('{}') in {:?} ?",
                request_id,
                self.request_book.clone(),
            ));
            // Make sure its our request
            let maybe_bucket_id = self.request_book.get(&request_id.clone());
            if maybe_bucket_id.is_none() {
                return None;
            }
            // Get bucketId
            bucket_id = maybe_bucket_id.unwrap().clone();
        }
        // drop request
        Some(bucket_id)
    }

    /// Received response from our request for the 'publish_list'
    /// For each data not already published, request it in order to publish it ourselves.
    fn priv_serve_HandleGetPublishingEntryListResult(
        &mut self,
        msg: &EntryListData,
    ) -> NetResult<()> {
        let bucket_id = self
            .priv_check_request(&msg.request_id)
            .expect("Not our request");
        self.log.d(&format!(
            "---- HandleGetPublishingDataListResult: bucket_id = '{}'",
            bucket_id,
        ));
        // Compare with already published list
        // For each data not already published, request it and publish it ourselves.
        let known_published_list = match self.published_entry_book.get(&bucket_id) {
            Some(list) => list.clone(),
            None => Vec::new(),
        };
        for entry_address in msg.entry_address_list.clone() {
            if known_published_list.contains(&entry_address) {
                continue;
            }
            let request_id = self.priv_create_request_with_bucket(&bucket_id);
            self.priv_send_one_with_bucket(
                &bucket_id,
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
        let bucket_id = self
            .priv_check_request(&msg.request_id)
            .expect("Not our request");
        self.log.d(&format!(
            "---- HandleGetHoldingEntryListResult: bucket_id = '{}'",
            bucket_id,
        ));
        // Compare with current stored_data_book
        // For each data not already holding, add it to stored_data_book?
        let known_stored_list = match self.stored_entry_book.get(&bucket_id) {
            Some(list) => list.clone(),
            None => Vec::new(),
        };
        for data_address in msg.entry_address_list.clone() {
            if known_stored_list.contains(&data_address) {
                continue;
            }
            bookkeep_address_with_bucket(
                &mut self.stored_entry_book,
                bucket_id.clone(),
                &data_address,
            );
        }
    }

    /// Received response from our request for the 'publish_list'
    /// For each data not already published, request it in order to publish it ourselves.
    fn priv_serve_HandleGetPublishingMetaListResult(
        &mut self,
        msg: &MetaListData,
    ) -> NetResult<()> {
        let bucket_id = self
            .priv_check_request(&msg.request_id)
            .expect("Not our request");
        self.log.d(&format!(
            "---- HandleGetPublishingMetaListResult: bucket_id = '{}'",
            bucket_id,
        ));
        // Compare with already published list
        // For each metadata not already published, request it and publish it ourselves.
        let known_published_meta_list = match self.published_meta_book.get(&bucket_id) {
            Some(list) => list.clone(),
            None => Vec::new(),
        };
        self.log.t(&format!(
            "known_published_meta_list = {:?}",
            known_published_meta_list
        ));

        let mut request_meta_key = Vec::new();
        for meta_tuple in msg.meta_list.clone() {
            let meta_id = into_meta_id(&meta_tuple);
            // dont send request for a known meta
            if known_published_meta_list.contains(&meta_id) {
                continue;
            }
            // dont send same request twice
            let meta_key = (meta_tuple.0.clone(), meta_tuple.1.clone());
            if request_meta_key.contains(&meta_key) {
                continue;
            }
            request_meta_key.push(meta_key);
            // send request for that meta_key
            let request_id = self.priv_create_request_with_bucket(&bucket_id);
            let fetch_meta = FetchMetaData {
                requester_agent_id: String::new(),
                request_id,
                dna_address: msg.dna_address.clone(),
                entry_address: meta_tuple.0,
                attribute: meta_tuple.1,
            };
            self.priv_send_one_with_bucket(
                &bucket_id,
                JsonProtocol::HandleFetchMeta(fetch_meta).into(),
            )?;
        }
        Ok(())
    }

    /// Received response from our request for the 'holding_meta_list'
    fn priv_serve_HandleGetHoldingMetaListResult(&mut self, msg: &MetaListData) {
        let bucket_id = self
            .priv_check_request(&msg.request_id)
            .expect("Not our request");
        self.log.d(&format!(
            "---- HandleGetHoldingMetaListResult: bucket_id = '{}'",
            bucket_id,
        ));
        // Compare with current stored_meta_book
        // For each data not already holding, add it to stored_meta_book?
        let known_stored_meta_list = match self.stored_meta_book.get(&bucket_id) {
            Some(list) => list.clone(),
            None => Vec::new(),
        };
        for meta_tuple in msg.meta_list.clone() {
            let meta_id = into_meta_id(&meta_tuple);
            if known_stored_meta_list.contains(&meta_id) {
                continue;
            }
            bookkeep_address_with_bucket(&mut self.stored_meta_book, bucket_id.clone(), &meta_id);
        }
    }
}
