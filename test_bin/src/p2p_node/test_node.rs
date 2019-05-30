#![allow(non_snake_case)]

use holochain_core_types::{
    cas::content::Address,
    hash::HashString,
};
use holochain_net::{
    connection::{
        json_protocol::{
            EntryListData, FetchEntryData, FetchEntryResultData,
            GetListData, JsonProtocol, MessageData, EntryAspectData,
            ProvidedEntryData, TrackDnaData, EntryData, QueryData,
            QueryResultData, GenericResultData,
        },
        net_connection::NetSend,
        protocol::Protocol,
        NetResult,
    },
    p2p_config::*,
    p2p_network::P2pNetwork,
    tweetlog::{TweetProxy, *},
};

use std::{
    collections::{HashMap, HashSet},
    convert::TryFrom,
    sync::mpsc,
    hash::Hash,
};

use super::{chain_store::ChainStore, ipc_config::create_ipc_config};

static TIMEOUT_MS: usize = 5000;

/// Conductor Mock of one agent with multiple DNAs
pub struct TestNode {
    // Need to hold the tempdir to keep it alive, otherwise we will get a dir error.
    _maybe_temp_dir: Option<tempfile::TempDir>,
    p2p_connection: P2pNetwork,
    receiver: mpsc::Receiver<Protocol>,
    pub config: P2pConfig,

    pub agent_id: Address,

    // my request logging
    request_log: Vec<String>,
    request_count: usize,

    // logging
    recv_msg_log: Vec<Protocol>,
    recv_dm_log: Vec<MessageData>,

    // datastores per dna
    chain_store_list: HashMap<Address, ChainStore>,
    tracked_dna_list: HashSet<Address>,

    pub current_dna: Option<Address>,

    pub logger: TweetProxy,

    is_network_ready: bool,
    pub p2p_binding: String,
}

/// Query logs
impl TestNode {
    /// Return number of JsonProtocol message this node has received
    pub fn count_recv_json_messages(&self) -> usize {
        let mut count = 0;
        for msg in self.recv_msg_log.clone() {
            if JsonProtocol::try_from(&msg).is_ok() {
                count += 1;
            };
        }
        count
    }

    /// Return the ith JSON message that this node has received and fullfills predicate
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

// Track
impl TestNode {
    pub fn track_current_dna(&mut self) -> NetResult<()> {
        assert!(self.current_dna.is_some());
        let current_dna = self.current_dna.clone().unwrap();
        self.track_dna(&current_dna, false)
    }

    pub fn track_dna(&mut self, dna_address: &Address, can_set_current: bool) -> NetResult<()> {
        if self.tracked_dna_list.contains(dna_address) {
            if can_set_current {
                self.set_current_dna(dna_address);
            }
            return Ok(());
        }
        let agent_id = self.agent_id.clone();
        let res = self.send(
            JsonProtocol::TrackDna(TrackDnaData {
                dna_address: dna_address.clone(),
                agent_id,
            })
            .into(),
        );
        if res.is_ok() {
            self.tracked_dna_list.insert(dna_address.clone());
            if !self.chain_store_list.contains_key(dna_address) {
                self.chain_store_list
                    .insert(dna_address.clone(), ChainStore::new(dna_address));
            }
            if can_set_current {
                self.set_current_dna(dna_address);
            }
        }
        res
    }

    pub fn untrack_current_dna(&mut self) -> NetResult<()> {
        assert!(self.current_dna.is_some());
        let current_dna = self.current_dna.clone().unwrap();
        let res = self.untrack_dna(&current_dna);
        if res.is_ok() {
            self.current_dna = None;
        }
        res
    }

    pub fn untrack_dna(&mut self, dna_address: &Address) -> NetResult<()> {
        if !self.tracked_dna_list.contains(dna_address) {
            return Ok(());
        }
        let agent_id = self.agent_id.clone();
        let res = self.send(
            JsonProtocol::UntrackDna(TrackDnaData {
                dna_address: dna_address.clone(),
                agent_id,
            })
            .into(),
        );
        if res.is_ok() {
            self.tracked_dna_list.remove(dna_address);
        }
        res
    }

    ///
    pub fn is_tracking(&self, dna_address: &Address) -> bool {
        self.tracked_dna_list.contains(dna_address)
    }

    ///
    pub fn set_current_dna(&mut self, dna_address: &Address) {
        if self.chain_store_list.contains_key(dna_address) {
            self.current_dna = Some(dna_address.clone());
        };
    }
}

///
impl TestNode {

    fn into_EntryData(entry_address: &Address, aspect_content_list: Vec<Vec<u8>>) -> EntryData {
        let mut aspect_list = Vec::new();
        for aspect_content in aspect_content_list {
            let hash = HashString::encode_from_bytes(aspect_content.as_slice(), Hash::SHA2256);
            aspect_list.push(EntryAspectData {
                aspect_address: hash,
                type_hint: "TestNode".to_string(),
                aspect: aspect_content,
                publish_ts: 42,
            });
        }
        EntryData {
            entry_address: entry_address.clone,
            aspect_list,
        }
    }

    pub fn author_entry(
        &mut self,
        entry_address: &Address,
        aspect_content_list: Vec<Vec<u8>>,
        can_broadcast: bool,
    ) -> NetResult<()> {
        assert!(self.current_dna.is_some());
        let current_dna = self.current_dna.clone().unwrap();
        let entry = TestNode::into_EntryData(entry_address, aspect_content_list);

        // bookkeep
        {
            let chain_store = self
                .chain_store_list
                .get_mut(&current_dna)
                .expect("No dna_store for this DNA");
            assert!(!chain_store.authored_entry_store.get(&entry_address).is_some());
            assert!(!chain_store.stored_entry_store.get(&entry_address).is_some());
            chain_store
                .authored_entry_store
                .insert_entry(&entry);
        }
        if can_broadcast {
            let msg_data = ProvidedEntryData {
                dna_address: current_dna,
                provider_agent_id: self.agent_id.clone(),
                entry: entry.clone(),
            };
            return self.send(JsonProtocol::PublishEntry(msg_data).into());
        }
        // Done
        Ok(())
    }

    pub fn hold_entry(&mut self,
                      entry_address: &Address,
                      aspect_content_list: Vec<Vec<u8>>) {
        assert!(self.current_dna.is_some());
        let current_dna = self.current_dna.clone().unwrap();
        let entry = TestNode::into_EntryData(entry_address, aspect_content_list);
        let chain_store = self
            .chain_store_list
            .get_mut(&current_dna)
            .expect("No dna_store for this DNA");
        assert!(!chain_store.authored_entry_store.get(&entry_address).is_some());
        assert!(!chain_store.stored_entry_store.get(&entry_address).is_some());
        chain_store
            .stored_entry_store
            .insert_entry(&entry);
    }
}

/// Query & send DirectMessage
impl TestNode {
    /// generate a new request_id
    fn generate_request_id(&mut self) -> String {
        self.request_count += 1;
        let request_id = format!("req_{}_{}", self.agent_id, self.request_count);
        self.request_log.push(request_id.clone());
        request_id
    }

    /// Node asks for some entry on the network.
    pub fn request_entry(&mut self, entry_address: Address) -> QueryData {
        assert!(self.current_dna.is_some());
        let current_dna = self.current_dna.clone().unwrap();
        let query_data = QueryData {
            dna_address: current_dna,
            entry_address,
            request_id: self.generate_request_id(),
            requester_agent_id: self.agent_id.clone(),
            query: vec![], // empty means give me the EntryData,
        };
        self.send(JsonProtocol::Query(query_data.clone()).into())
            .expect("Sending Query failed");
        query_data
    }

    /// Node asks for some entry on the network.
    pub fn reply_to_HandleQuery(&mut self, query: &QueryData) -> Result<QueryResultData, GenericResultData> {
        // Must be tracking DNA
        if !self.is_tracking(&query.dna_address) {
            let msg_data = GenericResultData {
                dna_address: query.dna_address.clone(),
                request_id: query.request_id.clone(),
                to_agent_id: query.requester_agent_id.clone(),
                result_info: "DNA is not tracked".as_bytes().to_vec(),
            };
            self.send(JsonProtocol::FailureResult(msg_data.clone()).into())
                .expect("Sending FailureResult failed");
            return Err(msg_data);
        }
        // Must be empty query
        if query.query == [] {
            let msg_data = GenericResultData {
                dna_address: query.dna_address.clone(),
                request_id: query.request_id.clone(),
                to_agent_id: query.requester_agent_id.clone(),
                result_info: "Unknown query request".as_bytes().to_vec(),
            };
            self.send(JsonProtocol::FailureResult(msg_data.clone()).into())
                .expect("Sending FailureResult failed");
            return Err(msg_data);
        }
        // Get Entry
        let maybe_store = self.chain_store_list.get(&query.dna_address);
        let query_result = match maybe_store {
            None => vec![],
            Some(chain_store) => {
                match chain_store.get_entry(&query.entry_address) {
                    None => vec![],
                    Some(entry) => entry.as_bytes().to_vec(),
                }
            },
        };
        // Send EntryData as binary
        let query_result_data = QueryResultData {
            dna_address: query.dna_address.clone(),
            entry_address: query.entry_address.clone(),
            request_id: query.request_id.clone(),
            requester_agent_id: query.requester_agent_id.clone(),
            responder_agent_id: self.agent_id.clone(),
            query_result,
        };
        self.send(JsonProtocol::HandleQueryResult(query_result_data.clone()).into())
            .expect("Sending Query failed");
        Ok(query_result_data)
    }

    /// Node sends Message on the network.
    pub fn send_direct_message(&mut self, to_agent_id: &Address, content: Vec<u8>) -> MessageData {
        println!("set_current_dna: {:?}", self.current_dna);
        assert!(self.current_dna.is_some());
        let current_dna = self.current_dna.clone().unwrap();
        let msg_data = MessageData {
            dna_address: current_dna,
            request_id: self.generate_request_id(),
            to_agent_id: to_agent_id.clone(),
            from_agent_id: self.agent_id.clone(),
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
        response_content: Vec<u8>,
    ) -> MessageData {
        assert!(self.current_dna.is_some());
        let current_dna = self.current_dna.clone().unwrap();
        assert_eq!(msg.dna_address, current_dna.clone());
        assert_eq!(msg.to_agent_id, self.agent_id);
        let response = MessageData {
            dna_address: current_dna.clone(),
            request_id: msg.request_id,
            to_agent_id: msg.from_agent_id.clone(),
            from_agent_id: self.agent_id.clone(),
            content: response_content,
        };
        self.send(JsonProtocol::HandleSendMessageResult(response.clone()).into())
            .expect("Sending HandleSendMessageResult failed");
        response
    }
}

// Replies
impl TestNode {
    // -- FETCH -- //

    /// Send a reponse to a FetchDhtData request
    pub fn reply_to_HandleFetchEntry(&mut self, request: &FetchEntryData) -> NetResult<()> {
        assert!(self.current_dna.is_some());
        let current_dna = self.current_dna.clone().unwrap();
        assert_eq!(request.dna_address, current_dna);
        // Create msg data
        let msg;
        {
            // Get data from local datastores
            let chain_store = self
                .chain_store_list
                .get_mut(&current_dna)
                .expect("No dna_store for this DNA");
            let mut maybe_entry = chain_store.authored_entry_store.get(&request.entry_address);
            if maybe_entry.is_none() {
                maybe_entry = chain_store.stored_entry_store.get(&request.entry_address);
            }
            // Send failure or success response
            msg = match maybe_entry.clone() {
                None => {
                    let msg_data = GenericResultData {
                        request_id: request.request_id.clone(),
                        dna_address: request.dna_address.clone(),
                        to_agent_id: request.requester_agent_id.clone(),
                        result_info: "Does not have the requested data".as_bytes().to_vec(),
                    };
                    JsonProtocol::FailureResult(msg_data).into()
                }
                Some(entry) => {
                    let msg_data = FetchEntryResultData {
                        dna_address: request.dna_address.clone(),
                        provider_agent_id: self.agent_id.clone(),
                        request_id: request.request_id.clone(),
                        entry: entry.clone(),
                    };
                    JsonProtocol::HandleFetchEntryResult(msg_data).into()
                }
            };
        }
        self.logger.t(&format!(
            "> ({}) reply_to_HandleFetchEntry() sending",
            self.agent_id,
        ));
        self.send(msg)
    }

    // -- LISTS -- //

    /// Reply to a HandleGetAuthoringEntryList request
    pub fn reply_to_HandleGetAuthoringEntryList(
        &mut self,
        request: &GetListData,
    ) -> NetResult<()> {
        assert!(self.current_dna.is_some());
        let current_dna = self.current_dna.clone().unwrap();
        assert_eq!(request.dna_address, current_dna);
        // Create msg data
        let msg;
        {
            let dna_store = self
                .chain_store_list
                .get_mut(&current_dna)
                .expect("No dna_store for this DNA");
            let entry_address_list = dna_store
                .authored_entry_store
                .iter()
                .map(|(k, _)| k.clone())
                .collect();
            msg = EntryListData {
                request_id: request.request_id.clone(),
                dna_address: request.dna_address.clone(),
                address_map: entry_address_list,
            };
        }
        self.send(JsonProtocol::HandleGetAuthoringEntryListResult(msg).into())
    }
    /// Look for the first HandleGetPublishingEntryList request received from network module and reply
    pub fn reply_to_first_HandleGetAuthoringEntryList(&mut self) {
        let request = self
            .find_recv_msg(
                0,
                Box::new(one_is!(JsonProtocol::HandleGetPublishingEntryList(_))),
            )
            .expect("Did not receive any HandleGetPublishingEntryList request");
        let get_entry_list_data = unwrap_to!(request => JsonProtocol::HandleGetPublishingEntryList);
        self.reply_to_HandleGetAuthoringEntryList(&get_entry_list_data)
            .expect("Reply to HandleGetPublishingEntryList failed.");
    }

    /// Reply to a HandleGetHoldingEntryList request
    pub fn reply_to_HandleGetHoldingEntryList(&mut self, request: &GetListData) -> NetResult<()> {
        assert!(self.current_dna.is_some());
        let current_dna = self.current_dna.clone().unwrap();
        assert_eq!(request.dna_address, current_dna);
        let msg;
        {
            let dna_store = self
                .chain_store_list
                .get_mut(&current_dna)
                .expect("No dna_store for this DNA");
            let entry_address_list = dna_store
                .stored_entry_store
                .iter()
                .map(|(k, _)| k.clone())
                .collect();
            msg = EntryListData {
                dna_address: request.dna_address.clone(),
                request_id: request.request_id.clone(),
                address_map: entry_address_list,
            };
        }
        self.send(JsonProtocol::HandleGetGossipingEntryListResult(msg).into())
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
}

impl TestNode {
    /// Private constructor
    #[cfg_attr(tarpaulin, skip)]
    pub fn new_with_config(
        agent_id_arg: &Address,
        config: &P2pConfig,
        _maybe_temp_dir: Option<tempfile::TempDir>,
    ) -> Self {
        log_dd!(
            "p2pnode",
            "new TestNode '{}' with config: {:?}",
            agent_id_arg,
            config
        );

        // use a mpsc channel for messaging between p2p connection and main thread
        let (sender, receiver) = mpsc::channel::<Protocol>();
        // create a new P2pNetwork instance with the handler that will send the received Protocol to a channel
        let agent_id = agent_id_arg.clone();
        let p2p_connection = P2pNetwork::new(
            Box::new(move |r| {
                log_tt!("p2pnode", "<<< ({}) handler: {:?}", agent_id_arg.to_string(), r);
                sender.send(r?)?;
                Ok(())
            }),
            &config,
        )
        .expect("Failed to create P2pNetwork");

        TestNode {
            _maybe_temp_dir,
            p2p_connection,
            receiver,
            config: config.clone(),
            agent_id,
            request_log: Vec::new(),
            request_count: 0,
            recv_msg_log: Vec::new(),
            recv_dm_log: Vec::new(),
            chain_store_list: HashMap::new(),
            tracked_dna_list: HashSet::new(),
            current_dna: None,
            logger: TweetProxy::new("p2pnode"),
            is_network_ready: false,
            p2p_binding: String::new(),
        }
    }

    #[cfg_attr(tarpaulin, skip)]
    pub fn is_network_ready(&self) -> bool {
        self.is_network_ready
    }

    /// Constructor for an in-memory P2P Network
    #[cfg_attr(tarpaulin, skip)]
    pub fn new_with_unique_memory_network(agent_id: &Address) -> Self {
        let config = P2pConfig::new_with_unique_memory_backend();
        return TestNode::new_with_config(agent_id, &config, None);
    }

    /// Constructor for an IPC node that uses an existing n3h process and a temp folder
    #[cfg_attr(tarpaulin, skip)]
    pub fn new_with_uri_ipc_network(agent_id: &Address, ipc_binding: &str) -> Self {
        let p2p_config = P2pConfig::default_ipc_uri(Some(ipc_binding));
        return TestNode::new_with_config(agent_id, &p2p_config, None);
    }

    /// Constructor for an IPC node that spawns and uses a n3h process and a temp folder
    #[cfg_attr(tarpaulin, skip)]
    pub fn new_with_spawn_ipc_network(
        agent_id: &Address,
        maybe_config_filepath: Option<&str>,
        maybe_end_user_config_filepath: Option<String>,
        bootstrap_nodes: Vec<String>,
        maybe_dir_path: Option<String>,
    ) -> Self {
        let (p2p_config, _maybe_temp_dir) = create_ipc_config(
            maybe_config_filepath,
            maybe_end_user_config_filepath,
            bootstrap_nodes,
            maybe_dir_path,
        );
        return TestNode::new_with_config(agent_id, &p2p_config, _maybe_temp_dir);
    }

    /// See if there is a message to receive, and log it
    /// return a JsonProtocol if the received message is of that type
    #[cfg_attr(tarpaulin, skip)]
    pub fn try_recv(&mut self) -> NetResult<JsonProtocol> {
        let data = self.receiver.try_recv()?;

        self.recv_msg_log.push(data.clone());

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
            Protocol::P2pReady => {
                let dbg_msg = format!("<< ({}) recv ** P2pReady **", self.agent_id);
                self.logger.d(&dbg_msg);
                self.is_network_ready = true;
                bail!("received P2pReady");
            }
            Protocol::Terminated => {
                let dbg_msg = format!("<< ({}) recv ** Terminated **", self.agent_id);
                self.logger.d(&dbg_msg);
                self.is_network_ready = false;
                bail!("received Terminated");
            }
            _ => {
                let dbg_msg = format!("<< ({}) recv <other>", self.agent_id);
                self.logger.t(&dbg_msg);
            }
        };

        match JsonProtocol::try_from(&data) {
            Ok(r) => {
                self.handle(r.clone());
                Ok(r)
            }
            Err(e) => {
                let s = format!("{:?}", e);
                if !s.contains("Empty") && !s.contains("Pong(PongData") {
                    self.logger.e(&format!(
                        "({}) ###### Received parse error: {} | data = {:?}",
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
        // Respond
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
            JsonProtocol::UntrackDna(_) => {
                panic!("Core should not receive UntrackDna message");
            }
            JsonProtocol::Connect(_) => {
                panic!("Core should not receive Connect message");
            }
            JsonProtocol::PeerConnected(_) => {
                // n/a
            }
            JsonProtocol::GetState => {
                panic!("Core should not receive GetState message");
            }
            JsonProtocol::GetStateResult(state) => {
                if !state.bindings.is_empty() {
                    self.p2p_binding = state.bindings[0].clone();
                }
            }
            JsonProtocol::GetDefaultConfig => {
                panic!("Core should not receive GetDefaultConfig message");
            }
            JsonProtocol::GetDefaultConfigResult(_) => {
                panic!("Core should not receive GetDefaultConfigResult message");
            }
            JsonProtocol::SetConfig(_) => {
                panic!("Core should not receive SetConfig message");
            }

            JsonProtocol::SendMessage(_) => {
                panic!("Core should not receive SendMessage message");
            }
            JsonProtocol::SendMessageResult(_) => {
                // n/a
            }
            JsonProtocol::HandleSendMessage(msg) => {
                // log the direct message sent to us
                self.recv_dm_log.push(msg);
            }
            JsonProtocol::HandleSendMessageResult(_msg) => {
                panic!("Core should not receive HandleSendMessageResult message");
            }

            JsonProtocol::HandleFetchEntry(_) => {
                // n/a
            }
            JsonProtocol::HandleFetchEntryResult(_) => {
                // n/a
            }

            JsonProtocol::PublishEntry(_msg) => {
                panic!("Core should not receive PublishDhtData message");
            }
            JsonProtocol::HandleStoreEntryAspect(msg) => {
                if self.is_tracking(&msg.dna_address) {
                    // Store data in local datastore
                    let mut chain_store = self
                        .chain_store_list
                        .get_mut(&msg.dna_address)
                        .expect("No dna_store for this DNA");
                    chain_store
                        .stored_entry_store
                        .insert_aspect(&msg.entry_address, &msg.entry_aspect);
                }
            }

            JsonProtocol::Query(_msg) => {
                panic!("Core should not receive Query message");
            }
            JsonProtocol::QueryResult(_msg) => {
                // n/a
            }
            JsonProtocol::HandleQuery(_msg) => {
                // n/a
            }
            JsonProtocol::HandleQueryResult(_msg) => {
                panic!("Core should not receive HandleQueryResult message");
            }

            // -- Publish & Hold data -- //
            JsonProtocol::HandleGetAuthoringEntryList(_) => {
                // n/a
            }
            JsonProtocol::HandleGetAuthoringEntryListResult(_) => {
                panic!("Core should not receive HandleGetPublishingDataListResult message");
            }
            JsonProtocol::HandleGetGossipingEntryList(_) => {
                // n/a
            }
            // Our request for the hold_list has returned
            JsonProtocol::HandleGetGossipingEntryListResult(_) => {
                panic!("Core should not receive HandleGetHoldingDataListResult message");
            }

            // ignore GetState, etc.
            _ => (),
        }
    }
}

impl NetSend for TestNode {
    /// send a Protocol message to the p2p network instance
    fn send(&mut self, data: Protocol) -> NetResult<()> {
        self.logger
            .d(&format!(">> ({}) send: {:?}", self.agent_id, data));
        self.p2p_connection.send(data)
    }
}
