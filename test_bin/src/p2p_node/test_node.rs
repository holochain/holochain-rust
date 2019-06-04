#![allow(non_snake_case)]

use holochain_core_types::{cas::content::Address, hash::HashString};
use holochain_net::{
    connection::{
        json_protocol::{
            EntryAspectData, EntryData, EntryListData, FetchEntryData, FetchEntryResultData,
            GenericResultData, GetListData, JsonProtocol, MessageData, ProvidedEntryData,
            QueryEntryData, QueryEntryResultData, TrackDnaData,
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
};

use super::{
    chain_store::ChainStore,
    create_config::{create_ipc_config, create_lib3h_config},
};
use crossbeam_channel::{unbounded, Receiver};
use holochain_net::connection::net_connection::NetHandler;
use lib3h_protocol::{
    data_types::DirectMessageData, protocol_client::Lib3hClientProtocol,
    protocol_server::Lib3hServerProtocol,
};
use multihash::Hash;

static TIMEOUT_MS: usize = 5000;

/// Conductor Mock of one agent with multiple DNAs
pub struct TestNode {
    // Need to hold the tempdir to keep it alive, otherwise we will get a dir error.
    _maybe_temp_dir: Option<tempfile::TempDir>,
    p2p_connection: P2pNetwork,
    receiver: Receiver<Protocol>,
    pub config: P2pConfig,

    pub agent_id: Address,

    // my request logging
    request_log: Vec<String>,
    request_count: usize,

    // logging
    recv_msg_log: Vec<Protocol>,

    // datastores per dna
    chain_store_list: HashMap<Address, ChainStore>,
    tracked_dna_list: HashSet<Address>,

    pub current_dna: Option<Address>,

    pub logger: TweetProxy,

    is_network_ready: bool,
    pub p2p_binding: String,
    is_json: bool,
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
    pub fn find_recv_json_msg(
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
        let protocol_msg: Protocol = if self.is_json {
            let track_dna_msg = TrackDnaData {
                dna_address: dna_address.clone(),
                agent_id,
            };
            JsonProtocol::TrackDna(track_dna_msg).into()
        } else {
            let track_dna_msg = lib3h_protocol::data_types::SpaceData {
                request_id: "leave_space_req".to_string(),
                space_address: dna_address.clone().to_string().into_bytes(),
                agent_id: agent_id.to_string().into_bytes(),
            };
            Lib3hClientProtocol::JoinSpace(track_dna_msg).into()
        };
        println!("TestNode.track_dna(): {:?}", protocol_msg);
        let res = self.send(protocol_msg);
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
        let protocol_msg: Protocol = if self.is_json {
            let track_dna_msg = TrackDnaData {
                dna_address: dna_address.clone(),
                agent_id,
            };
            JsonProtocol::UntrackDna(track_dna_msg).into()
        } else {
            let leave_space_msg = lib3h_protocol::data_types::SpaceData {
                request_id: "leave_space_req".to_string(),
                space_address: dna_address.clone().to_string().into_bytes(),
                agent_id: agent_id.to_string().into_bytes(),
            };
            Lib3hClientProtocol::LeaveSpace(leave_space_msg).into()
        };
        let res = self.send(protocol_msg);
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
    /// Convert an aspect_content_list into an EntryData
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
            entry_address: entry_address.clone(),
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
            let res = chain_store.author_entry(&entry);
            // Entry is known, try authoring each aspect instead
            if res.is_err() {
                let mut success = false;
                for aspect in &entry.aspect_list {
                    let aspect_res = chain_store.author_aspect(&entry.entry_address, aspect);
                    if aspect_res.is_ok() {
                        success = true;
                    }
                }
                if !success {
                    return Err(format_err!("Authoring of all aspects failed."));
                }
            }
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

    pub fn hold_entry(
        &mut self,
        entry_address: &Address,
        aspect_content_list: Vec<Vec<u8>>,
    ) -> NetResult<()> {
        assert!(self.current_dna.is_some());
        let current_dna = self.current_dna.clone().unwrap();
        let entry = TestNode::into_EntryData(entry_address, aspect_content_list);
        let chain_store = self
            .chain_store_list
            .get_mut(&current_dna)
            .expect("No dna_store for this DNA");
        let res = chain_store.hold_entry(&entry);
        // Entry is known, try authoring each aspect instead
        if res.is_err() {
            let mut success = false;
            for aspect in entry.aspect_list {
                let aspect_res = chain_store.hold_aspect(&entry.entry_address, &aspect);
                if aspect_res.is_ok() {
                    success = true;
                }
            }
            if !success {
                return Err(format_err!("Storing of all aspects failed."));
            }
        }
        // Done
        Ok(())
    }
}

/// Query & Fetch
impl TestNode {
    /// generate a new request_id
    fn generate_request_id(&mut self) -> String {
        self.request_count += 1;
        let request_id = format!("req_{}_{}", self.agent_id, self.request_count);
        self.request_log.push(request_id.clone());
        request_id
    }

    /// Node asks for some entry on the network.
    pub fn request_entry(&mut self, entry_address: Address) -> QueryEntryData {
        assert!(self.current_dna.is_some());
        let current_dna = self.current_dna.clone().unwrap();
        let query_data = QueryEntryData {
            dna_address: current_dna,
            entry_address,
            request_id: self.generate_request_id(),
            requester_agent_id: self.agent_id.clone(),
            query: vec![], // empty means give me the EntryData,
        };
        self.send(JsonProtocol::QueryEntry(query_data.clone()).into())
            .expect("Sending Query failed");
        query_data
    }

    ///
    pub fn reply_to_HandleQueryEntry(
        &mut self,
        query: &QueryEntryData,
    ) -> Result<QueryEntryResultData, GenericResultData> {
        // Must be empty query
        if !query.query.is_empty() {
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
        // Convert query to fetch
        let fetch = FetchEntryData {
            dna_address: query.dna_address.clone(),
            request_id: query.request_id.clone(),
            provider_agent_id: query.requester_agent_id.clone(),
            entry_address: query.entry_address.clone(),
            aspect_address_list: None,
        };
        // HandleFetchEntry
        let fetch_res = self.reply_to_HandleFetchEntry_inner(&fetch);
        if let Err(res) = fetch_res {
            self.send(JsonProtocol::FailureResult(res.clone()).into())
                .expect("Sending FailureResult failed");
            return Err(res);
        }
        // Convert query to fetch
        let query_res = QueryEntryResultData {
            dna_address: query.dna_address.clone(),
            entry_address: query.entry_address.clone(),
            request_id: query.request_id.clone(),
            requester_agent_id: query.requester_agent_id.clone(),
            responder_agent_id: self.agent_id.clone(),
            query_result: bincode::serialize(&fetch_res.unwrap().entry).unwrap(),
        };
        self.send(JsonProtocol::HandleQueryEntryResult(query_res.clone()).into())
            .expect("Sending FailureResult failed");
        return Ok(query_res);
    }

    ///
    pub fn reply_to_HandleFetchEntry(
        &mut self,
        fetch: &FetchEntryData,
    ) -> Result<FetchEntryResultData, GenericResultData> {
        let fetch_res = self.reply_to_HandleFetchEntry_inner(fetch);
        let json_msg = match fetch_res.clone() {
            Err(res) => JsonProtocol::FailureResult(res),
            Ok(fetch) => JsonProtocol::HandleFetchEntryResult(fetch),
        };
        self.send(json_msg.into()).expect("Sending failed");
        fetch_res
    }

    /// Node asks for some entry on the network.
    fn reply_to_HandleFetchEntry_inner(
        &mut self,
        fetch: &FetchEntryData,
    ) -> Result<FetchEntryResultData, GenericResultData> {
        // Must be tracking DNA
        if !self.is_tracking(&fetch.dna_address) {
            let msg_data = GenericResultData {
                dna_address: fetch.dna_address.clone(),
                request_id: fetch.request_id.clone(),
                to_agent_id: fetch.provider_agent_id.clone(),
                result_info: "DNA is not tracked".as_bytes().to_vec(),
            };
            return Err(msg_data);
        }
        // Get Entry
        let maybe_store = self.chain_store_list.get(&fetch.dna_address);
        let maybe_entry = match maybe_store {
            None => None,
            Some(chain_store) => chain_store.get_entry(&fetch.entry_address),
        };
        // No entry, send failure
        if maybe_entry.is_none() {
            let msg_data = GenericResultData {
                dna_address: fetch.dna_address.clone(),
                request_id: fetch.request_id.clone(),
                to_agent_id: fetch.provider_agent_id.clone(),
                result_info: "No entry found".as_bytes().to_vec(),
            };
            return Err(msg_data);
        }
        // Send EntryData as binary
        let fetch_result_data = FetchEntryResultData {
            dna_address: fetch.dna_address.clone(),
            provider_agent_id: fetch.provider_agent_id.clone(),
            request_id: fetch.request_id.clone(),
            entry: maybe_entry.unwrap(),
        };
        Ok(fetch_result_data)
    }
}
impl TestNode {
    /// Node sends Message on the network.
    pub fn send_direct_message(&mut self, to_agent_id: &Address, content: Vec<u8>) -> String {
        println!("set_current_dna: {:?}", self.current_dna);
        assert!(self.current_dna.is_some());
        let dna_address = self.current_dna.clone().unwrap();
        let request_id = self.generate_request_id();
        let from_agent_id = self.agent_id.to_string();

        let p = if self.is_json {
            let msg_data = MessageData {
                dna_address,
                from_agent_id: self.agent_id.clone(),
                request_id: self.generate_request_id(),
                to_agent_id: to_agent_id.clone(),
                content,
            };
            JsonProtocol::SendMessage(msg_data.clone()).into()
        } else {
            let msg_data = DirectMessageData {
                space_address: dna_address.to_string().into_bytes(),
                request_id: request_id.clone(),
                to_agent_id: to_agent_id.to_string().into_bytes(),
                from_agent_id: from_agent_id.to_string().into_bytes(),
                content,
            };
            Lib3hClientProtocol::SendDirectMessage(msg_data.clone()).into()
        };
        self.send(p).expect("Sending SendMessage failed");
        request_id
    }

    /// Node sends Message on the network.
    pub fn send_reponse_json(&mut self, msg: MessageData, response_content: Vec<u8>) {
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
    }

    /// Node sends Message on the network.
    pub fn send_reponse_lib3h(
        &mut self,
        msg: DirectMessageData,
        response_content: serde_json::Value,
    ) {
        assert!(self.current_dna.is_some());
        let current_dna = self.current_dna.clone().unwrap();
        assert_eq!(
            msg.space_address,
            current_dna.clone().to_string().into_bytes()
        );
        assert_eq!(
            msg.to_agent_id,
            self.agent_id.clone().to_string().into_bytes()
        );
        let response = DirectMessageData {
            space_address: current_dna.clone().to_string().into_bytes(),
            request_id: msg.request_id,
            to_agent_id: msg.from_agent_id.clone(),
            from_agent_id: self.agent_id.to_string().into_bytes(),
            content: response_content.to_string().into_bytes(),
        };
        self.send(Lib3hClientProtocol::HandleSendDirectMessageResult(response.clone()).into())
            .expect("Sending HandleSendMessageResult failed");
    }
}

/// Reply LISTS
impl TestNode {
    /// Reply to a HandleGetAuthoringEntryList request
    pub fn reply_to_HandleGetAuthoringEntryList(&mut self, request: &GetListData) -> NetResult<()> {
        assert!(self.current_dna.is_some());
        let current_dna = self.current_dna.clone().unwrap();
        assert_eq!(request.dna_address, current_dna);
        // Create msg data
        let msg;
        {
            let authored_entry_store = self
                .chain_store_list
                .get_mut(&current_dna)
                .expect("No chain_store for this DNA")
                .get_authored_store();
            let mut entry_address_list = HashMap::new();
            for (entry_address, entry_map) in authored_entry_store {
                let aspect_map = entry_map
                    .iter()
                    .map(|(a_address, _)| a_address.clone())
                    .collect();
                entry_address_list.insert(entry_address, aspect_map);
            }
            msg = EntryListData {
                request_id: request.request_id.clone(),
                dna_address: request.dna_address.clone(),
                address_map: entry_address_list,
                provider_agent_id: self.agent_id.clone(),
            };
        }
        self.send(JsonProtocol::HandleGetAuthoringEntryListResult(msg).into())
    }
    /// Look for the first HandleGetAuthoringEntryList request received from network module and reply
    pub fn reply_to_first_HandleGetAuthoringEntryList(&mut self) {
        let request = self
            .find_recv_json_msg(
                0,
                Box::new(one_is!(JsonProtocol::HandleGetAuthoringEntryList(_))),
            )
            .expect("Did not receive any HandleGetAuthoringEntryList request");
        let get_entry_list_data = unwrap_to!(request => JsonProtocol::HandleGetAuthoringEntryList);
        self.reply_to_HandleGetAuthoringEntryList(&get_entry_list_data)
            .expect("Reply to HandleGetAuthoringEntryList failed.");
    }

    /// Reply to a HandleGetHoldingEntryList request
    pub fn reply_to_HandleGetHoldingEntryList(&mut self, request: &GetListData) -> NetResult<()> {
        assert!(self.current_dna.is_some());
        let current_dna = self.current_dna.clone().unwrap();
        assert_eq!(request.dna_address, current_dna);
        let msg;
        {
            let stored_entry_store = self
                .chain_store_list
                .get_mut(&current_dna)
                .expect("No chain_store for this DNA")
                .get_stored_store();
            let mut entry_address_list = HashMap::new();
            for (entry_address, entry_map) in stored_entry_store {
                let aspect_map = entry_map
                    .iter()
                    .map(|(a_address, _)| a_address.clone())
                    .collect();
                entry_address_list.insert(entry_address, aspect_map);
            }
            msg = EntryListData {
                request_id: request.request_id.clone(),
                dna_address: request.dna_address.clone(),
                address_map: entry_address_list,
                provider_agent_id: self.agent_id.clone(),
            };
        }
        self.send(JsonProtocol::HandleGetGossipingEntryListResult(msg).into())
    }
    /// Look for the first HandleGetHoldingEntryList request received from network module and reply
    pub fn reply_to_first_HandleGetHoldingEntryList(&mut self) {
        let request = self
            .find_recv_json_msg(
                0,
                Box::new(one_is!(JsonProtocol::HandleGetGossipingEntryList(_))),
            )
            .expect("Did not receive a HandleGetHoldingEntryList request");
        // extract request data
        let get_list_data = unwrap_to!(request => JsonProtocol::HandleGetGossipingEntryList);
        // reply
        self.reply_to_HandleGetHoldingEntryList(&get_list_data)
            .expect("Reply to HandleGetHoldingEntryList failed.");
    }
}

impl TestNode {
    /// Private constructor
    #[cfg_attr(tarpaulin, skip)]
    pub fn new_with_config(
        agent_id_arg: Address,
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
        let (sender, receiver) = unbounded::<Protocol>();
        // create a new P2pNetwork instance with the handler that will send the received Protocol to a channel
        let agent_id = agent_id_arg.clone();
        let p2p_connection = P2pNetwork::new(
            NetHandler::new(Box::new(move |r| {
                log_tt!("p2pnode", "<<< ({}) handler: {:?}", agent_id_arg, r);
                sender.send(r?)?;
                Ok(())
            })),
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
            chain_store_list: HashMap::new(),
            tracked_dna_list: HashSet::new(),
            current_dna: None,
            logger: TweetProxy::new("p2pnode"),
            is_network_ready: false,
            p2p_binding: String::new(),
            is_json: config.backend_kind != P2pBackendKind::LIB3H,
        }
    }

    #[cfg_attr(tarpaulin, skip)]
    pub fn is_network_ready(&self) -> bool {
        self.is_network_ready
    }

    /// Constructor for an in-memory P2P Network
    #[cfg_attr(tarpaulin, skip)]
    pub fn new_with_unique_memory_network(agent_id: Address) -> Self {
        let config = P2pConfig::new_with_unique_memory_backend();
        return TestNode::new_with_config(agent_id, &config, None);
    }

    /// Constructor for an IPC node that uses an existing n3h process and a temp folder
    #[cfg_attr(tarpaulin, skip)]
    pub fn new_with_uri_ipc_network(agent_id: Address, ipc_binding: &str) -> Self {
        let p2p_config = P2pConfig::default_ipc_uri(Some(ipc_binding));
        return TestNode::new_with_config(agent_id, &p2p_config, None);
    }

    /// Constructor for an IPC node that uses an existing n3h process and a temp folder
    #[cfg_attr(tarpaulin, skip)]
    pub fn new_with_lib3h(
        agent_id: Address,
        maybe_config_filepath: Option<&str>,
        maybe_end_user_config_filepath: Option<String>,
        bootstrap_nodes: Vec<String>,
        maybe_dir_path: Option<String>,
    ) -> Self {
        let (p2p_config, _maybe_temp_dir) = create_lib3h_config(
            maybe_config_filepath,
            maybe_end_user_config_filepath,
            bootstrap_nodes,
            maybe_dir_path,
        );
        return TestNode::new_with_config(agent_id, &p2p_config, _maybe_temp_dir);
    }

    /// Constructor for an IPC node that spawns and uses a n3h process and a temp folder
    #[cfg_attr(tarpaulin, skip)]
    pub fn new_with_spawn_ipc_network(
        agent_id: Address,
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

    #[cfg_attr(tarpaulin, skip)]
    pub fn try_recv(&mut self) -> NetResult<Protocol> {
        let data = self.receiver.try_recv()?;

        self.recv_msg_log.push(data.clone());
        Ok(data)
    }

    /// See if there is a message to receive, and log it
    /// return a JsonProtocol if the received message is of that type
    #[cfg_attr(tarpaulin, skip)]
    pub fn try_recv_json(&mut self) -> NetResult<JsonProtocol> {
        let data = self.try_recv()?;

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
                self.handle_json(r.clone());
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

    /// See if there is a message to receive, and log it
    /// return a JsonProtocol if the received message is of that type
    #[cfg_attr(tarpaulin, skip)]
    pub fn try_recv_lib3h(&mut self) -> NetResult<Lib3hServerProtocol> {
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

        match Lib3hServerProtocol::try_from(&data) {
            Ok(r) => {
                self.handle_lib3h(r.clone());
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
    pub fn wait_HandleFetchEntry_and_reply(&mut self) -> bool {
        let maybe_request = self.wait_json(Box::new(one_is!(JsonProtocol::HandleFetchEntry(_))));
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

    /// wait to receive a HandleFetchEntry request and automatically reply
    /// return true if a HandleFetchEntry has been received
    pub fn wait_HandleQueryEntry_and_reply(&mut self) -> bool {
        let maybe_request = self.wait_json(Box::new(one_is!(JsonProtocol::HandleQueryEntry(_))));
        if maybe_request.is_none() {
            return false;
        }
        let request = maybe_request.unwrap();
        // extract msg data
        let query_data = unwrap_to!(request => JsonProtocol::HandleQueryEntry);
        // Respond
        self.reply_to_HandleQueryEntry(&query_data)
            .expect("Reply to HandleFetchEntry should work");
        true
    }

    /// Wait for receiving a message corresponding to predicate until timeout is reached
    pub fn wait_json_with_timeout(
        &mut self,
        predicate: Box<dyn Fn(&JsonProtocol) -> bool>,
        timeout_ms: usize,
    ) -> Option<JsonProtocol> {
        let mut time_ms: usize = 0;
        loop {
            let mut did_something = false;

            if let Ok(p2p_msg) = self.try_recv_json() {
                self.logger.i(&format!(
                    "({})::wait_json() - received: {:?}",
                    self.agent_id, p2p_msg
                ));
                did_something = true;
                if predicate(&p2p_msg) {
                    self.logger
                        .i(&format!("({})::wait_json() - match", self.agent_id));
                    return Some(p2p_msg);
                } else {
                    self.logger
                        .i(&format!("({})::wait_json() - NO match", self.agent_id));
                }
            }

            if !did_something {
                std::thread::sleep(std::time::Duration::from_millis(10));
                time_ms += 10;
                if time_ms > timeout_ms {
                    self.logger
                        .i(&format!("({})::wait_json() has TIMEOUT", self.agent_id));
                    return None;
                }
            }
        }
    }

    /// Wait for receiving a message corresponding to predicate
    /// hard coded timeout
    #[cfg_attr(tarpaulin, skip)]
    pub fn wait_json(
        &mut self,
        predicate: Box<dyn Fn(&JsonProtocol) -> bool>,
    ) -> Option<JsonProtocol> {
        self.wait_json_with_timeout(predicate, TIMEOUT_MS)
    }

    /// Wait for receiving a message corresponding to predicate
    /// hard coded timeout
    #[cfg_attr(tarpaulin, skip)]
    pub fn wait_lib3h(
        &mut self,
        predicate: Box<dyn Fn(&Lib3hServerProtocol) -> bool>,
    ) -> Option<Lib3hServerProtocol> {
        self.wait_lib3h_with_timeout(predicate, TIMEOUT_MS)
    }

    /// Wait for receiving a message corresponding to predicate until timeout is reached
    pub fn wait_lib3h_with_timeout(
        &mut self,
        predicate: Box<dyn Fn(&Lib3hServerProtocol) -> bool>,
        timeout_ms: usize,
    ) -> Option<Lib3hServerProtocol> {
        let mut time_ms: usize = 0;
        loop {
            let mut did_something = false;

            if let Ok(p2p_msg) = self.try_recv() {
                if let Protocol::Lib3hServer(lib3h_msg) = p2p_msg {
                    self.logger.i(&format!(
                        "({})::wait_lib3h() - received: {:?}",
                        self.agent_id, lib3h_msg
                    ));
                    did_something = true;
                    if predicate(&lib3h_msg) {
                        self.logger
                            .i(&format!("({})::wait_lib3h() - match", self.agent_id));
                        return Some(lib3h_msg);
                    } else {
                        self.logger
                            .i(&format!("({})::wait_lib3h() - NO match", self.agent_id));
                    }
                }
            }

            if !did_something {
                std::thread::sleep(std::time::Duration::from_millis(10));
                time_ms += 10;
                if time_ms > timeout_ms {
                    self.logger
                        .i(&format!("({})::wait_lib3h() has TIMEOUT", self.agent_id));
                    return None;
                }
            }
        }
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
    fn handle_lib3h(&mut self, lib3h_msg: Lib3hServerProtocol) {
        match lib3h_msg {
            Lib3hServerProtocol::SuccessResult(_msg) => {
                // FIXME
            }
            Lib3hServerProtocol::FailureResult(_msg) => {
                // FIXME
            }
            Lib3hServerProtocol::Connected(_msg) => {
                // FIXME
            }
            Lib3hServerProtocol::Disconnected(_msg) => {
                // FIXME
            }
            Lib3hServerProtocol::SendDirectMessageResult(_msg) => {
                // FIXME
            }
            Lib3hServerProtocol::HandleSendDirectMessage(_msg) => {
                // FIXME
            }
            Lib3hServerProtocol::FetchEntryResult(_msg) => {
                // FIXME
            }
            Lib3hServerProtocol::HandleFetchEntry(_msg) => {
                // FIXME
            }
            Lib3hServerProtocol::HandleStoreEntry(_msg) => {
                // FIXME
            }
            Lib3hServerProtocol::HandleDropEntry(_msg) => {
                // FIXME
            }
            Lib3hServerProtocol::HandleGetPublishingEntryList(_msg) => {
                // FIXME
            }
            Lib3hServerProtocol::HandleGetHoldingEntryList(_msg) => {
                // FIXME
            }
        }
    }
    /// handle all types of json message
    #[cfg_attr(tarpaulin, skip)]
    fn handle_json(&mut self, json_msg: JsonProtocol) {
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
            JsonProtocol::HandleSendMessage(_msg) => {
                // log the direct message sent to us
                // FIXME
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
                    let _ = chain_store.hold_aspect(&msg.entry_address, &msg.entry_aspect);
                }
            }

            JsonProtocol::QueryEntry(_msg) => {
                panic!("Core should not receive Query message");
            }
            JsonProtocol::QueryEntryResult(_msg) => {
                // n/a
            }
            JsonProtocol::HandleQueryEntry(_msg) => {
                // n/a
            }
            JsonProtocol::HandleQueryEntryResult(_msg) => {
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
