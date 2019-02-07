#![allow(non_snake_case)]

use holochain_core_types::{cas::content::Address, hash::HashString};
use holochain_net::{
    p2p_config::*,
    p2p_network::P2pNetwork,
    tweetlog::{TweetProxy, *},
};
use holochain_net_connection::{
    json_protocol::{
        DhtMetaData, EntryData, EntryListData, FailureResultData, FetchEntryData,
        FetchEntryResultData, FetchMetaData, FetchMetaResultData, GetListData, JsonProtocol,
        MessageData, MetaKey, MetaListData, MetaTuple,
    },
    net_connection::NetSend,
    protocol::Protocol,
    NetResult,
};
use multihash::Hash;
use std::{collections::HashMap, convert::TryFrom, sync::mpsc};

static TIMEOUT_MS: usize = 5000;

pub type MetaStoreValue = serde_json::Value;

pub struct MetaStore {
    // TODO: Changed once meta is only Addresses
    // pub meta_store: HashMap<MetaKey, HashSet<Address>>,
    store: HashMap<MetaKey, HashMap<Address, serde_json::Value>>,
}

impl MetaStore {
    pub fn new() -> Self {
        MetaStore {
            store: HashMap::new(),
        }
    }

    /// Check if this value is already stored
    pub fn has(&self, meta_key: MetaKey, v: &MetaStoreValue) -> bool {
        let hash = HashString::encode_from_str(&v.to_string(), Hash::SHA2256);
        let maybe_map = self.store.get(&meta_key);
        if maybe_map.is_none() {
            return false;
        }
        maybe_map.unwrap().get(&hash).is_some()
    }

    ///
    pub fn insert(&mut self, meta_key: MetaKey, v: MetaStoreValue) {
        let hash = HashString::encode_from_str(&v.to_string(), Hash::SHA2256);
        if let None = self.store.get_mut(&meta_key) {
            let mut map = HashMap::new();
            log_tt!(
                "metastore",
                "MetaStore: first content for '{:?}' = {} | {}",
                meta_key,
                v,
                hash,
            );
            map.insert(hash, v);
            self.store.insert(meta_key, map);
        } else {
            if let Some(map) = self.store.get_mut(&meta_key) {
                assert!(map.get(&hash).is_none());
                log_tt!(
                    "metastore",
                    "MetaStore: adding content for '{:?}' = {} | {}",
                    meta_key,
                    v,
                    hash,
                );
                map.insert(hash, v);
            };
        };
    }

    /// Get all values for a meta_key as a vec
    pub fn get(&self, meta_key: MetaKey) -> Vec<serde_json::Value> {
        let maybe_metas = self.store.get(&meta_key);
        let metas = match maybe_metas.clone() {
            Some(map) => map.clone(),
            // if meta not found return empty list (will make the aggregation easier)
            None => HashMap::new(),
        };
        let res = metas.iter().map(|(_, v)| v.clone()).collect();
        res
    }

    /// Get all values stored
    pub fn get_all(&self) -> Vec<MetaTuple> {
        let mut meta_list: Vec<MetaTuple> = Vec::new();
        for (meta_key, meta_map) in self.store.clone() {
            for (_, v) in meta_map {
                meta_list.push((meta_key.0.clone(), meta_key.1.clone(), v));
            }
        }
        meta_list
    }
}

/// Core Mock
pub struct P2pNode {
    // Need to hold the tempdir to keep it alive, otherwise we will get a dir error.
    _maybe_temp_dir: Option<tempfile::TempDir>,
    p2p_connection: P2pNetwork,
    receiver: mpsc::Receiver<Protocol>,
    pub config: P2pConfig,

    pub agent_id: String,

    // my request logging
    request_log: Vec<String>,
    request_count: usize,

    // logging
    recv_msg_log: Vec<Protocol>,
    recv_dm_log: Vec<MessageData>,

    // datastores for one dna
    // TODO: Have a datastore per dna ; perhaps in a CoreMock struct
    pub dna_address: Address,
    pub entry_store: HashMap<Address, serde_json::Value>,

    pub meta_store: MetaStore,
    pub authored_entry_store: HashMap<Address, serde_json::Value>,
    pub authored_meta_store: MetaStore,

    pub logger: TweetProxy,
}

// Search logs
// return the ith message that fullfills the predicate
impl P2pNode {
    pub fn find_recv_msg(
        &self,
        ith: usize,
        predicate: Box<dyn Fn(&JsonProtocol) -> bool>,
    ) -> Option<JsonProtocol> {
        let mut count = 0;
        for msg in self.recv_msg_log.clone() {
            let json_msg = match JsonProtocol::try_from(&msg) {
                Ok(r) => r,
                Err(_) => continue,
            };
            if predicate(&json_msg) {
                if count == ith {
                    return Some(json_msg);
                }
                count += 1;
            }
        }
        None
    }
}

// Publish, hold
impl P2pNode {
    pub fn author_entry(
        &mut self,
        entry_address: &Address,
        entry_content: &serde_json::Value,
        can_publish: bool,
    ) -> NetResult<()> {
        assert!(!self.authored_entry_store.get(&entry_address).is_some());
        assert!(!self.entry_store.get(&entry_address).is_some());
        self.authored_entry_store
            .insert(entry_address.clone(), entry_content.clone());
        if can_publish {
            let msg_data = EntryData {
                dna_address: self.dna_address.clone(),
                provider_agent_id: self.agent_id.clone(),
                entry_address: entry_address.clone(),
                entry_content: entry_content.clone(),
            };
            return self.send(JsonProtocol::PublishEntry(msg_data).into());
        }
        // Done
        Ok(())
    }

    pub fn author_meta(
        &mut self,
        entry_address: &Address,
        attribute: &str,
        link_entry_address: &serde_json::Value,
        can_publish: bool,
    ) -> NetResult<MetaKey> {
        let meta_key = (entry_address.clone(), attribute.to_string());

        // bookkeep
        {
            // Must not already have meta
            assert!(!self.meta_store.has(meta_key.clone(), link_entry_address));
            self.authored_meta_store
                .insert(meta_key.clone(), link_entry_address.clone());
        }
        // publish it
        if can_publish {
            let msg_data = DhtMetaData {
                dna_address: self.dna_address.clone(),
                provider_agent_id: self.agent_id.clone(),
                entry_address: entry_address.clone(),
                attribute: attribute.to_string(),
                content_list: vec![link_entry_address.clone()],
            };
            let res = self.send(JsonProtocol::PublishMeta(msg_data).into());
            return res.map(|_| meta_key);
        }
        // Done
        Ok(meta_key.clone())
    }

    pub fn hold_entry(&mut self, entry_address: &Address, entry_content: &serde_json::Value) {
        assert!(!self.authored_entry_store.get(&entry_address).is_some());
        assert!(!self.entry_store.get(&entry_address).is_some());
        self.entry_store
            .insert(entry_address.clone(), entry_content.clone());
    }

    pub fn hold_meta(
        &mut self,
        entry_address: &Address,
        attribute: &str,
        link_entry_address: &serde_json::Value,
    ) {
        let meta_key = (entry_address.clone(), attribute.to_string());
        // Must not already have meta
        assert!(!self
            .authored_meta_store
            .has(meta_key.clone(), link_entry_address));
        self.meta_store.insert(meta_key, link_entry_address.clone());
    }
}

/// fetch & sendMessage
impl P2pNode {
    /// generate a new request_id
    fn generate_request_id(&mut self) -> String {
        self.request_count += 1;
        let request_id = format!("req_{}_{}", self.agent_id, self.request_count);
        self.request_log.push(request_id.clone());
        request_id
    }

    /// Node asks for some entry on the network.
    pub fn request_entry(&mut self, entry_address: Address) -> FetchEntryData {
        let fetch_data = FetchEntryData {
            request_id: self.generate_request_id(),
            dna_address: self.dna_address.clone(),
            requester_agent_id: self.agent_id.clone(),
            entry_address,
        };
        self.send(JsonProtocol::FetchEntry(fetch_data.clone()).into())
            .expect("Sending FetchEntry failed");
        fetch_data
    }

    /// Node asks for some meta on the network.
    pub fn request_meta(&mut self, entry_address: Address, attribute: String) -> FetchMetaData {
        let fetch_meta = FetchMetaData {
            request_id: self.generate_request_id(),
            dna_address: self.dna_address.clone(),
            requester_agent_id: self.agent_id.to_string(),
            entry_address,
            attribute,
        };
        self.send(JsonProtocol::FetchMeta(fetch_meta.clone()).into())
            .expect("Sending FetchMeta failed");
        fetch_meta
    }

    /// Node sends Message on the network.
    pub fn send_message(&mut self, to_agent_id: String, content: serde_json::Value) -> MessageData {
        let msg_data = MessageData {
            dna_address: self.dna_address.clone(),
            from_agent_id: self.agent_id.to_string(),
            request_id: self.generate_request_id(),
            to_agent_id,
            content,
        };
        self.send(JsonProtocol::SendMessage(msg_data.clone()).into())
            .expect("Sending SendMessage failed");
        msg_data
    }

    /// Node sends Message on the network.
    pub fn send_reponse(
        &mut self,
        msg: MessageData,
        response_content: serde_json::Value,
    ) -> MessageData {
        assert_eq!(msg.dna_address, self.dna_address);
        assert_eq!(msg.to_agent_id, self.agent_id);
        let response = MessageData {
            dna_address: self.dna_address.clone(),
            from_agent_id: self.agent_id.to_string(),
            request_id: msg.request_id,
            to_agent_id: msg.from_agent_id.clone(),
            content: response_content,
        };
        self.send(JsonProtocol::HandleSendMessageResult(response.clone()).into())
            .expect("Sending HandleSendMessageResult failed");
        response
    }
}

// Replies
impl P2pNode {
    // -- FETCH -- //

    /// Send a reponse to a FetchDhtData request
    pub fn reply_to_HandleFetchEntry(&mut self, request: &FetchEntryData) -> NetResult<()> {
        assert_eq!(request.dna_address, self.dna_address);
        let msg;
        {
            // Get data from local datastores
            let mut maybe_data = self.authored_entry_store.get(&request.entry_address);
            if maybe_data.is_none() {
                maybe_data = self.entry_store.get(&request.entry_address);
            }
            // Send failure or success response
            msg = match maybe_data.clone() {
                None => {
                    let msg_data = FailureResultData {
                        request_id: request.request_id.clone(),
                        dna_address: request.dna_address.clone(),
                        to_agent_id: request.requester_agent_id.clone(),
                        error_info: json!("Does not have the requested data"),
                    };
                    JsonProtocol::FailureResult(msg_data).into()
                }
                Some(data) => {
                    let msg_data = FetchEntryResultData {
                        request_id: request.request_id.clone(),
                        requester_agent_id: request.requester_agent_id.clone(),
                        dna_address: request.dna_address.clone(),
                        provider_agent_id: self.agent_id.clone(),
                        entry_address: request.entry_address.clone(),
                        entry_content: data.clone(),
                    };
                    JsonProtocol::HandleFetchEntryResult(msg_data).into()
                }
            };
        }
        self.send(msg)
    }

    /// Send a reponse to a FetchDhtMetaData request
    pub fn reply_to_HandleFetchMeta(&mut self, request: &FetchMetaData) -> NetResult<()> {
        assert_eq!(request.dna_address, self.dna_address);
        let msg;
        {
            // Get meta from local datastores
            let meta_key = (request.entry_address.clone(), request.attribute.clone());
            let mut metas = self.authored_meta_store.get(meta_key.clone());
            if metas.is_empty() {
                metas = self.meta_store.get(meta_key);
            }
            self.logger.t(&format!("metas = {:?}", metas));
            msg = FetchMetaResultData {
                request_id: request.request_id.clone(),
                requester_agent_id: request.requester_agent_id.clone(),
                dna_address: request.dna_address.clone(),
                provider_agent_id: self.agent_id.clone(),
                entry_address: request.entry_address.clone(),
                attribute: request.attribute.clone(),
                content_list: metas,
            };
        }
        self.send(JsonProtocol::HandleFetchMetaResult(msg).into())
    }

    // -- LISTS -- //

    /// Reply to a HandleGetPublishingEntryList request
    pub fn reply_to_HandleGetPublishingEntryList(
        &mut self,
        request: &GetListData,
    ) -> NetResult<()> {
        assert_eq!(request.dna_address, self.dna_address);
        let entry_address_list = self
            .authored_entry_store
            .iter()
            .map(|(k, _)| k.clone())
            .collect();
        let msg = EntryListData {
            entry_address_list: entry_address_list,
            request_id: request.request_id.clone(),
            dna_address: request.dna_address.clone(),
        };
        self.send(JsonProtocol::HandleGetPublishingEntryListResult(msg).into())
    }
    /// Look for the first HandleGetPublishingEntryList request received from network module and reply
    pub fn reply_to_first_HandleGetPublishingEntryList(&mut self) {
        let request = self
            .find_recv_msg(
                0,
                Box::new(one_is!(JsonProtocol::HandleGetPublishingEntryList(_))),
            )
            .expect("Did not receive any HandleGetPublishingEntryList request");
        let get_entry_list_data = unwrap_to!(request => JsonProtocol::HandleGetPublishingEntryList);
        self.reply_to_HandleGetPublishingEntryList(&get_entry_list_data)
            .expect("Reply to HandleGetPublishingEntryList failed.");
    }

    /// Reply to a HandleGetPublishingMetaList request
    pub fn reply_to_HandleGetPublishingMetaList(&mut self, request: &GetListData) -> NetResult<()> {
        assert_eq!(request.dna_address, self.dna_address);
        let msg = MetaListData {
            request_id: request.request_id.clone(),
            dna_address: request.dna_address.clone(),
            meta_list: self.authored_meta_store.get_all(),
        };
        self.send(JsonProtocol::HandleGetPublishingMetaListResult(msg).into())
    }
    /// Look for the first HandleGetPublishingMetaList request received from network module and reply
    pub fn reply_to_first_HandleGetPublishingMetaList(&mut self) {
        self.logger.t(&format!(
            "--- HandleGetPublishingMetaList: {}",
            self.agent_id
        ));
        let request = self
            .find_recv_msg(
                0,
                Box::new(one_is!(JsonProtocol::HandleGetPublishingMetaList(_))),
            )
            .expect("Did not receive a HandleGetPublishingMetaList request");
        let get_meta_list_data = unwrap_to!(request => JsonProtocol::HandleGetPublishingMetaList);
        self.reply_to_HandleGetPublishingMetaList(&get_meta_list_data)
            .expect("Reply to HandleGetPublishingMetaList failed.");
    }

    /// Reply to a HandleGetHoldingEntryList request
    pub fn reply_to_HandleGetHoldingEntryList(&mut self, request: &GetListData) -> NetResult<()> {
        assert_eq!(request.dna_address, self.dna_address);
        let entry_address_list = self.entry_store.iter().map(|(k, _)| k.clone()).collect();
        let msg = EntryListData {
            request_id: request.request_id.clone(),
            dna_address: request.dna_address.clone(),
            entry_address_list: entry_address_list,
        };
        self.send(JsonProtocol::HandleGetHoldingEntryListResult(msg).into())
    }
    /// Look for the first HandleGetHoldingEntryList request received from network module and reply
    pub fn reply_to_first_HandleGetHoldingEntryList(&mut self) {
        let request = self
            .find_recv_msg(
                0,
                Box::new(one_is!(JsonProtocol::HandleGetHoldingEntryList(_))),
            )
            .expect("Did not receive a HandleGetHoldingEntryList request");
        // extract request data
        let get_list_data = unwrap_to!(request => JsonProtocol::HandleGetHoldingEntryList);
        // reply
        self.reply_to_HandleGetHoldingEntryList(&get_list_data)
            .expect("Reply to HandleGetHoldingEntryList failed.");
    }

    /// Reply to a HandleGetHoldingMetaList request
    pub fn reply_to_HandleGetHoldingMetaList(&mut self, request: &GetListData) -> NetResult<()> {
        assert_eq!(request.dna_address, self.dna_address);
        let msg = MetaListData {
            request_id: request.request_id.clone(),
            dna_address: request.dna_address.clone(),
            meta_list: self.meta_store.get_all(),
        };
        self.send(JsonProtocol::HandleGetHoldingMetaListResult(msg).into())
    }
    /// Look for the first HandleGetHoldingMetaList request received from network module and reply
    pub fn reply_to_first_HandleGetHoldingMetaList(&mut self) {
        let request = self
            .find_recv_msg(
                0,
                Box::new(one_is!(JsonProtocol::HandleGetHoldingMetaList(_))),
            )
            .expect("Did not receive a HandleGetHoldingMetaList request");
        // extract request data
        let get_list_data = unwrap_to!(request => JsonProtocol::HandleGetHoldingMetaList);
        // reply
        self.reply_to_HandleGetHoldingMetaList(&get_list_data)
            .expect("Reply to HandleGetHoldingMetaList failed.");
    }
}

impl P2pNode {
    /// Private constructor
    #[cfg_attr(tarpaulin, skip)]
    pub fn new_with_config(
        agent_id_arg: String,
        dna_address: Address,
        config: &P2pConfig,
        _maybe_temp_dir: Option<tempfile::TempDir>,
    ) -> Self {
        // use a mpsc channel for messaging between p2p connection and main thread
        let (sender, receiver) = mpsc::channel::<Protocol>();
        // create a new P2pNetwork instance with the handler that will send the received Protocol to a channel
        let agent_id = agent_id_arg.clone();
        let p2p_connection = P2pNetwork::new(
            Box::new(move |r| {
                log_tt!("p2pnode", "<<< ({}) handler: {:?}", agent_id_arg, r);
                sender.send(r?)?;
                Ok(())
            }),
            &config,
        )
        .expect("Failed to create P2pNetwork");

        P2pNode {
            _maybe_temp_dir,
            p2p_connection,
            receiver,
            config: config.clone(),
            agent_id,
            request_log: Vec::new(),
            request_count: 0,
            recv_msg_log: Vec::new(),
            recv_dm_log: Vec::new(),
            dna_address,
            entry_store: HashMap::new(),
            meta_store: MetaStore::new(),
            authored_entry_store: HashMap::new(),
            authored_meta_store: MetaStore::new(),
            logger: TweetProxy::new("p2pnode"),
        }
    }

    /// Constructor for an in-memory P2P Network
    #[cfg_attr(tarpaulin, skip)]
    pub fn new_with_unique_memory_network(agent_id: String, dna_address: Address) -> Self {
        let config = P2pConfig::new_with_unique_memory_backend();
        return P2pNode::new_with_config(agent_id, dna_address, &config, None);
    }

    /// Constructor for an IPC node that uses an existing n3h process and a temp folder
    #[cfg_attr(tarpaulin, skip)]
    pub fn new_with_uri_ipc_network(
        agent_id: String,
        dna_address: Address,
        ipc_binding: &str,
    ) -> Self {
        let p2p_config = P2pConfig::default_ipc_uri(Some(ipc_binding));
        return P2pNode::new_with_config(agent_id, dna_address, &p2p_config, None);
    }

    /// Constructor for an IPC node that spawns and uses a n3h process and a temp folder
    #[cfg_attr(tarpaulin, skip)]
    pub fn new_with_spawn_ipc_network(
        agent_id: String,
        dna_address: Address,
        n3h_path: &str,
        maybe_config_filepath: Option<&str>,
        bootstrap_nodes: Vec<String>,
    ) -> Self {
        let (p2p_config, temp_dir) =
            create_ipc_config(n3h_path, maybe_config_filepath, bootstrap_nodes);
        return P2pNode::new_with_config(agent_id, dna_address, &p2p_config, Some(temp_dir));
    }

    /// See if there is a message to receive, and log it
    /// return a JsonProtocol if the received message is of that type
    #[cfg_attr(tarpaulin, skip)]
    pub fn try_recv(&mut self) -> NetResult<JsonProtocol> {
        let data = self.receiver.try_recv()?;
        // logging depending on received type
        match data {
            Protocol::NamedBinary(_) => {
                let dbg_msg = format!("<< ({}) recv: {:?}", self.agent_id, data);
                self.logger.d(&dbg_msg);
            }
            Protocol::Json(_) => {
                let dbg_msg = format!("<< ({}) recv: {:?}", self.agent_id, data);
                self.logger.d(&dbg_msg);
            }
            _ => {
                let dbg_msg = format!("<< ({}) recv <other>", self.agent_id);
                self.logger.t(&dbg_msg);
            }
        };

        self.recv_msg_log.push(data.clone());

        match JsonProtocol::try_from(&data) {
            Ok(r) => {
                self.handle(r.clone());
                Ok(r)
            }
            Err(e) => {
                let s = format!("{:?}", e);
                if !s.contains("Empty") && !s.contains("Pong(PongData") {
                    self.logger.e(&format!(
                        "({}) ###### Received parse error: {} {:?}",
                        self.agent_id, s, data,
                    ));
                }
                Err(e)
            }
        }
    }

    /// recv messages until timeout is reached
    /// returns the number of messages it received during listening period
    /// timeout is reset after a message is received
    #[cfg_attr(tarpaulin, skip)]
    pub fn listen(&mut self, timeout_ms: usize) -> usize {
        let mut count: usize = 0;
        let mut time_ms: usize = 0;
        loop {
            let mut has_recved = false;

            if let Ok(p2p_msg) = self.try_recv() {
                self.logger.t(&format!(
                    "({})::listen() - received: {:?}",
                    self.agent_id, p2p_msg,
                ));
                has_recved = true;
                time_ms = 0;
                count += 1;
            }
            if !has_recved {
                std::thread::sleep(std::time::Duration::from_millis(10));
                time_ms += 10;
                if time_ms > timeout_ms {
                    return count;
                }
            }
        }
    }

    /// wait to receive a HandleFetchEntry request and automatically reply
    /// return true if a HandleFetchEntry has been received
    #[allow(non_snake_case)]
    pub fn wait_HandleFetchEntry_and_reply(&mut self) -> bool {
        let maybe_request = self.wait(Box::new(one_is!(JsonProtocol::HandleFetchEntry(_))));
        if maybe_request.is_none() {
            return false;
        }
        let request = maybe_request.unwrap();
        // extract msg data
        let fetch_data = unwrap_to!(request => JsonProtocol::HandleFetchEntry);
        // Alex responds: should send entry data back
        self.reply_to_HandleFetchEntry(&fetch_data)
            .expect("Reply to HandleFetchEntry should work");
        true
    }

    /// wait to receive a HandleFetchMeta request and automatically reply
    /// return true if a HandleFetchMeta has been received
    #[allow(non_snake_case)]
    pub fn wait_HandleFetchMeta_and_reply(&mut self) -> bool {
        let maybe_request = self.wait(Box::new(one_is!(JsonProtocol::HandleFetchMeta(_))));
        if maybe_request.is_none() {
            return false;
        }
        let request = maybe_request.unwrap();
        // extract msg data
        let fetch_meta = unwrap_to!(request => JsonProtocol::HandleFetchMeta);
        // Alex responds: should send entry data back
        self.reply_to_HandleFetchMeta(&fetch_meta)
            .expect("Reply to HandleFetchMeta should work");
        true
    }

    /// Wait for receiving a message corresponding to predicate until timeout is reached
    pub fn wait_with_timeout(
        &mut self,
        predicate: Box<dyn Fn(&JsonProtocol) -> bool>,
        timeout_ms: usize,
    ) -> Option<JsonProtocol> {
        let mut time_ms: usize = 0;
        loop {
            let mut did_something = false;

            if let Ok(p2p_msg) = self.try_recv() {
                self.logger.i(&format!(
                    "({})::wait() - received: {:?}",
                    self.agent_id, p2p_msg
                ));
                did_something = true;
                if predicate(&p2p_msg) {
                    self.logger
                        .i(&format!("({})::wait() - match", self.agent_id));
                    return Some(p2p_msg);
                } else {
                    self.logger
                        .i(&format!("({})::wait() - NO match", self.agent_id));
                }
            }

            if !did_something {
                std::thread::sleep(std::time::Duration::from_millis(10));
                time_ms += 10;
                if time_ms > timeout_ms {
                    self.logger
                        .i(&format!("({})::wait() has TIMEOUT", self.agent_id));
                    return None;
                }
            }
        }
    }

    /// Wait for receiving a message corresponding to predicate
    /// hard coded timeout
    #[cfg_attr(tarpaulin, skip)]
    pub fn wait(&mut self, predicate: Box<dyn Fn(&JsonProtocol) -> bool>) -> Option<JsonProtocol> {
        self.wait_with_timeout(predicate, TIMEOUT_MS)
    }

    // Stop node
    #[cfg_attr(tarpaulin, skip)]
    pub fn stop(self) {
        self.p2p_connection
            .stop()
            .expect("Failed to stop p2p connection properly");
    }

    /// Getter of the endpoint of its connection
    #[cfg_attr(tarpaulin, skip)]
    pub fn endpoint(&self) -> String {
        self.p2p_connection.endpoint()
    }

    /// handle all types of json message
    #[cfg_attr(tarpaulin, skip)]
    fn handle(&mut self, json_msg: JsonProtocol) {
        match json_msg {
            JsonProtocol::SuccessResult(_msg) => {
                // n/a
            }
            JsonProtocol::FailureResult(_msg) => {
                // n/a
            }
            JsonProtocol::TrackDna(_) => {
                panic!("Core should not receive TrackDna message");
            }
            JsonProtocol::Connect(_) => {
                panic!("Core should not receive Connect message");
            }
            JsonProtocol::PeerConnected(_msg) => {
                // n/a
            }
            JsonProtocol::SendMessage(_) => {
                panic!("Core should not receive SendMessage message");
            }
            JsonProtocol::SendMessageResult(_) => {
                // n/a
            }
            JsonProtocol::HandleSendMessage(msg) => {
                assert_eq!(msg.dna_address, self.dna_address);
                // log the direct message sent to us
                self.recv_dm_log.push(msg);
            }
            JsonProtocol::HandleSendMessageResult(_msg) => {
                panic!("Core should not receive HandleSendMessageResult message");
            }

            JsonProtocol::FetchEntry(_) => {
                panic!("Core should not receive FetchDhtData message");
            }
            JsonProtocol::FetchEntryResult(_) => {
                // n/a
            }
            JsonProtocol::HandleFetchEntry(msg) => {
                assert_eq!(msg.dna_address, self.dna_address);
                self.reply_to_HandleFetchEntry(&msg)
                    .expect("Should reply to a HandleFetchEntry");
            }
            JsonProtocol::HandleFetchEntryResult(_msg) => {
                // n/a
            }

            JsonProtocol::PublishEntry(_msg) => {
                panic!("Core should not receive PublishDhtData message");
            }
            JsonProtocol::HandleStoreEntry(msg) => {
                assert_eq!(msg.dna_address, self.dna_address);
                // Store data in local datastore
                self.entry_store
                    .insert(msg.entry_address, msg.entry_content);
            }
            JsonProtocol::HandleDropEntry(msg) => {
                assert_eq!(msg.dna_address, self.dna_address);
                // Remove data in local datastore
                self.entry_store.remove(&msg.entry_address);
            }

            JsonProtocol::FetchMeta(_msg) => {
                panic!("Core should not receive FetchDhtMeta message");
            }
            JsonProtocol::FetchMetaResult(_msg) => {
                // n/a
            }
            JsonProtocol::HandleFetchMeta(msg) => {
                assert_eq!(msg.dna_address, self.dna_address);
                self.reply_to_HandleFetchMeta(&msg)
                    .expect("Should reply to a HandleFetchMeta");
            }
            JsonProtocol::HandleFetchMetaResult(_msg) => {
                // n/a
            }

            JsonProtocol::PublishMeta(_msg) => {
                panic!("Core should not receive PublishDhtMeta message");
            }
            JsonProtocol::HandleStoreMeta(msg) => {
                assert_eq!(msg.dna_address, self.dna_address);
                // Store data in local datastore
                let meta_key = (msg.entry_address, msg.attribute);
                for content in msg.content_list {
                    self.meta_store.insert(meta_key.clone(), content);
                }
            }
            // TODO
            //            JsonProtocol::HandleDropMeta(msg) => {
            //                assert_eq!(msg.dna_address, self.dna_address);
            //                // Remove data in local datastore
            //                self.meta_store.remove(&(msg.entry_address, msg.attribute));
            //            }

            // -- Publish & Hold data -- //
            JsonProtocol::HandleGetPublishingEntryList(msg) => {
                assert_eq!(msg.dna_address, self.dna_address);
                // n/a
            }
            JsonProtocol::HandleGetPublishingEntryListResult(_) => {
                panic!("Core should not receive HandleGetPublishingDataListResult message");
            }
            JsonProtocol::HandleGetHoldingEntryList(msg) => {
                assert_eq!(msg.dna_address, self.dna_address);
                // n/a
            }
            // Our request for the hold_list has returned
            JsonProtocol::HandleGetHoldingEntryListResult(_) => {
                panic!("Core should not receive HandleGetHoldingDataListResult message");
            }

            // -- Publish & Hold meta -- //
            JsonProtocol::HandleGetPublishingMetaList(msg) => {
                assert_eq!(msg.dna_address, self.dna_address);
                // n/a
            }
            JsonProtocol::HandleGetPublishingMetaListResult(_) => {
                panic!("Core should not receive HandleGetPublishingMetaListResult message");
            }
            JsonProtocol::HandleGetHoldingMetaList(msg) => {
                assert_eq!(msg.dna_address, self.dna_address);
                // n/a
            }
            // Our request for the hold_list has returned
            JsonProtocol::HandleGetHoldingMetaListResult(_) => {
                panic!("Core should not receive HandleGetHoldingMetaListResult message");
            }
            // ignore GetState, etc.
            _ => (),
        }
    }
}

impl NetSend for P2pNode {
    /// send a Protocol message to the p2p network instance
    fn send(&mut self, data: Protocol) -> NetResult<()> {
        self.logger
            .d(&format!(">> ({}) send: {:?}", self.agent_id, data));
        self.p2p_connection.send(data)
    }
}

//--------------------------------------------------------------------------------------------------
// create_ipc_config
//--------------------------------------------------------------------------------------------------

/// Create an P2pConfig for an IPC node that uses n3h and a temp folder
#[cfg_attr(tarpaulin, skip)]
fn create_ipc_config(
    n3h_path: &str,
    maybe_config_filepath: Option<&str>,
    bootstrap_nodes: Vec<String>,
) -> (P2pConfig, tempfile::TempDir) {
    // Create temp directory
    let dir_ref = tempfile::tempdir().expect("Failed to created a temp directory.");
    let dir = dir_ref.path().to_string_lossy().to_string();

    log_i!("create_ipc_config() dir = {}", dir);

    // Create config
    let config = match maybe_config_filepath {
        Some(filepath) => {
            log_d!("filepath = {}", filepath);
            // Get config from file
            let p2p_config = P2pConfig::from_file(filepath);
            assert_eq!(p2p_config.backend_kind, P2pBackendKind::IPC);
            // complement missing fields
            serde_json::from_value(json!({
            "backend_kind": String::from(p2p_config.backend_kind),
            "backend_config":
            {
                "socketType": p2p_config.backend_config["socketType"],
                "bootstrapNodes": bootstrap_nodes,
                "spawn":
                {
                    "cmd": p2p_config.backend_config["spawn"]["cmd"],
                    "args": [
                        format!("{}/packages/n3h/bin/n3h", n3h_path)
                    ],
                    "workDir": dir.clone(),
                    "env": {
                        "N3H_MODE": p2p_config.backend_config["spawn"]["env"]["N3H_MODE"],
                        "N3H_WORK_DIR": dir.clone(),
                        "N3H_IPC_SOCKET": p2p_config.backend_config["spawn"]["env"]["N3H_IPC_SOCKET"],
                    }
                },
            }})).expect("Failled making valid P2pConfig with filepath")
        }
        None => {
            // use default config
            serde_json::from_value(json!({
            "backend_kind": "IPC",
            "backend_config":
            {
                "socketType": "zmq",
                "bootstrapNodes": bootstrap_nodes,
                "spawn":
                {
                    "cmd": "node",
                    "args": [
                        format!("{}/packages/n3h/bin/n3h", n3h_path)
                    ],
                    "workDir": dir.clone(),
                    "env": {
                        "N3H_MODE": "HACK",
                        "N3H_WORK_DIR": dir.clone(),
                        "N3H_IPC_SOCKET": "tcp://127.0.0.1:*",
                }
            },
            }}))
            .expect("Failled making valid default P2pConfig")
        }
    };
    return (config, dir_ref);
}
