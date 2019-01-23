//! provides fake in-memory p2p worker for use in scenario testing

#![allow(non_snake_case)]

use holochain_core_types::cas::content::Address;
use holochain_net_connection::{
    json_protocol::{
        DhtData, DhtMetaData, FailureResultData, FetchDhtData, FetchDhtMetaData, JsonProtocol,
        MessageData, PeerData, HandleDhtResultData, HandleDhtMetaResultData, GetListData,
        HandleListResultData,
    },
    protocol::Protocol,
    NetResult,
};
use std::{
    collections::{hash_map::Entry, HashMap},
    convert::TryFrom,
    sync::{mpsc, Mutex, RwLock},
};
use std::collections::HashSet;

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

//fn uncat_dna_agent(bucket_id: &str) -> (Address, &str) {
//    let v: Vec<&str> = bucket_id.split("::").collect();
//    assert_eq!(v.len(), 2);
//    (Addresss::from(v[0]), v[1])
//}

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
    // request book: request_id -> bucket_id
    request_book: HashMap<String, String>,
    // published data book: bucket_id -> entry_addresses
    published_data_list: HashMap<String, Vec<Address>>,
    // stored data book: bucket_id -> entry_addresses
    stored_data_list: HashMap<String, Vec<Address>>,
    // published meta data book: bucket_id -> entry_addresses?
    published_metadata_list: HashMap<String, Vec<Address>>,
    // stored meta data book: bucket_id -> entry_addresses?
    stored_metadata_list: HashMap<String, Vec<Address>>,
    // used for making unique request ids
    request_count: usize,
}

impl InMemoryServer {
    /// create a new in-memory network server
    pub fn new(name: String) -> Self {
        //println!("NEW InMemoryServer '{}'", name.clone());
        Self {
            name,
            senders: HashMap::new(),
            senders_by_dna: HashMap::new(),
            client_count: 0,
            request_book: HashMap::new(),
            published_data_list: HashMap::new(),
            stored_data_list: HashMap::new(),
            published_metadata_list: HashMap::new(),
            stored_metadata_list: HashMap::new(),
            request_count: 0,
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

    fn priv_generate_request_id(&mut self) -> String {
        self.request_count += 1;
        format!("req_{}", self.request_count)
    }

    fn priv_create_request(&mut self, dna_address: &Address, agent_id: &str) -> String {
        let bucket_id = cat_dna_agent(dna_address, agent_id);
        let req_id = self.priv_generate_request_id();
            self.request_book.insert(req_id.clone(), bucket_id);
        req_id
    }

    fn priv_create_request_with_bucket(&mut self, bucket_id: &str) -> String {
        let req_id = self.priv_generate_request_id();
        self.request_book.insert(req_id.clone(), bucket_id.to_string());
        req_id
    }

    fn priv_drop_request(&mut self, id: &str) -> bool {
        self.request_book.remove(id).is_some()
    }

    fn priv_book_published_data(
        &mut self,
        dna_address: &Address,
        agent_id: &str,
        data_address: &Address,
    ) {
        let bucket_id = cat_dna_agent(dna_address, agent_id);
        // Append existing address list if there is one
        {
            let maybe_vec_address = self.published_data_list.get_mut(&bucket_id);
            if let Some(vec_address) = maybe_vec_address {
                vec_address.push(data_address.clone());
                return;
            }
        }
        // None: Create and add a new address list
        let vec = vec![data_address.clone()];
        self.published_data_list.insert(bucket_id, vec);
    }


    /// register a data handler with the server (for message routing)
    pub fn register(
        &mut self,
        dna_address: &Address,
        agent_id: &str,
        sender: mpsc::Sender<Protocol>,
    ) -> NetResult<()> {
        let mut can_request_publish_list= false;
        {
            self.senders
                .insert(cat_dna_agent(dna_address, agent_id), sender.clone());
            match self.senders_by_dna.entry(dna_address.to_owned()) {
                Entry::Occupied(mut e) => {
                    e.get_mut().push(sender.clone());
                }
                Entry::Vacant(e) => {
                    e.insert(vec![sender.clone()]);
                    can_request_publish_list = true;
                }
            };
        }
        if can_request_publish_list {
            // Request this agent's published data
            let request_id = self.priv_create_request(dna_address, agent_id);
            self.priv_send_one(
                dna_address,
                agent_id,
                JsonProtocol::HandleGetPublishingDataList(GetListData {
                    request_id,
                    dna_address: dna_address.clone(),
                }).into(),
            )?;
        }
        Ok(())
    }

    /// process a message sent by a node to the "network"
    pub fn serve(&mut self, data: Protocol) -> NetResult<()> {
         //Debugging code (do not remove)
                println!(
                    ">>>> InMemoryServer '{}' recv: {:?}",
                    self.name.clone(),
                    data
                );
        if let Ok(json_msg) = JsonProtocol::try_from(&data) {
            match json_msg {
                JsonProtocol::TrackDna(msg) => {
                    // Notify all Peers connected to this DNA of a new Peer connection.
                    self.priv_send_all(
                        &msg.dna_address.clone(),
                        JsonProtocol::PeerConnected(PeerData {
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

                // Our request for the publish_list has returned
                JsonProtocol::HandleGetPublishingDataListResult(msg) => {
                    let mut bucket_id;
                    {
                        println!(
                            "---- InMemoryServer::HandleGetPublishingDataListResult: req_id = '{}' || {:?}",
                            msg.request_id,
                            self.request_book.clone(),
                        );

                        // Make sure its our request
                        let maybe_bucket_id = self.request_book.get(&msg.request_id);
                        if maybe_bucket_id.is_none() {
                            return Ok(());
                        }
                        // Get bucketId
                        bucket_id = maybe_bucket_id.unwrap().clone();
                    }
                    // drop request
                    self.priv_drop_request(&msg.request_id);

                    println!(
                        "---- InMemoryServer::HandleGetPublishingDataListResult: bucket_id = '{}'",
                        bucket_id,
                    );

                    // Compare with already published list
                    // For each data not already published, request it and publish it ourselves.
                    let known_published_list = match self.published_data_list.get(&bucket_id) {
                        Some(list) => list.clone(),
                        None => Vec::new(),
                    };
                    let dna_address = msg.dna_address.clone();
                    for data_address in msg.data_address_list {
                        let has_published_data = known_published_list.contains(&data_address);
                        if has_published_data {
                            continue;
                        }
                        let request_id = self.priv_create_request_with_bucket(&bucket_id);
                        self.priv_send_one_with_bucket(
                            &bucket_id,
                            JsonProtocol::HandleFetchDhtData(FetchDhtData {
                                requester_agent_id: String::new(),
                                request_id,
                                dna_address: dna_address.clone(),
                                data_address,
                            }).into(),
                        )?;
                    }
                }
                _ => (),
            }
        }
        Ok(())
    }

    // -- private -- //

    /// send a message to the appropriate channel based on dna_address::to_agent_id
    fn priv_send_one_with_bucket(
        &mut self,
        bucket_id: &str,
        data: Protocol,
    ) -> NetResult<()> {
        let maybe_sender = self.senders.get_mut(bucket_id);
        if maybe_sender.is_none() {
            //println!("#### InMemoryServer '{}' error: No sender channel found", self.name.clone());
            return Err(format_err!(
                "No sender channel found ({})",
                self.name.clone()
            ));
        }
        let sender = maybe_sender.unwrap();
        //Debugging code (do not remove)
        println!(
            "<<<< InMemoryServer '{}' send: {:?}",
            self.name.clone(),
            data
        );
        sender.send(data)?;
        Ok(())
    }
    /// send a message to the appropriate channel based on dna_address::to_agent_id
    fn priv_send_one(
        &mut self,
        dna_address: &Address,
        to_agent_id: &str,
        data: Protocol,
    ) -> NetResult<()> {
        let bucked_id = cat_dna_agent(dna_address, to_agent_id);
        self.priv_send_one_with_bucket(&bucked_id, data)
    }

    /// send a message to all nodes connected with this dna address
    fn priv_send_all(&mut self, dna_address: &Address, data: Protocol) -> NetResult<()> {
        if let Some(arr) = self.senders_by_dna.get_mut(dna_address) {
             //Debugging code (do not remove)
                        println!(
                            "<<<< InMemoryServer '{}' send all: {:?} ({})",
                            self.name.clone(),
                            data.clone(),
                            dna_address.clone()
                        );
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
        self.priv_book_published_data(&msg.dna_address, &msg.provider_agent_id, &msg.data_address);
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

    /// send back a response to a request for dht data
    fn priv_serve_HandleFetchDhtDataResult(&mut self, msg: &HandleDhtResultData) -> NetResult<()> {
        // if its from our own request do a publish
        if self.priv_drop_request(&msg.request_id) {
            let dht_data = DhtData {
                dna_address: msg.dna_address.clone(),
                provider_agent_id: msg.provider_agent_id.clone(),
            data_address: msg.data_address.clone(),
            data_content: msg.data_content.clone(),
            };
            self.priv_serve_PublishDhtData(&dht_data);
            return Ok(());
        }
        // otherwise just send back to requester
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
    fn priv_serve_HandleFetchDhtMetaResult(&mut self, msg: &HandleDhtMetaResultData) -> NetResult<()> {
        self.priv_send_one(
            &msg.dna_address,
            &msg.requester_agent_id,
            JsonProtocol::FetchDhtMetaResult(msg.clone()).into(),
        )?;
        Ok(())
    }
}
