#![allow(non_snake_case)]

use holochain_net::{
    connection::{net_connection::NetSend, NetResult},
    p2p_config::*,
    p2p_network::P2pNetwork,
    tweetlog::{TweetProxy, *},
};

use lib3h_protocol::{
    data_types::{
        DirectMessageData, EntryAspectData, EntryData, EntryListData, FetchEntryData,
        FetchEntryResultData, GenericResultData, GetListData, ProvidedEntryData, QueryEntryData,
        QueryEntryResultData, SpaceData,
    },
    protocol_client::Lib3hClientProtocol,
    protocol_server::Lib3hServerProtocol,
};

use holochain_persistence_api::{cas::content::Address, hash::HashString};

use std::collections::{HashMap, HashSet};

use super::{
    chain_store::ChainStore,
    create_config::{create_ipc_config, create_lib3h_config},
};
use crossbeam_channel::{unbounded, Receiver};
use holochain_net::connection::net_connection::NetHandler;
use multihash::Hash;

static TIMEOUT_MS: usize = 5000;

/// Conductor Mock of one agent with multiple DNAs
pub struct TestNode {
    // Need to hold the tempdir to keep it alive, otherwise we will get a dir error.
    _maybe_temp_dir: Option<tempfile::TempDir>,
    p2p_connection: P2pNetwork,
    receiver: Receiver<Lib3hServerProtocol>,
    pub config: P2pConfig,

    pub agent_id: Address,

    // my request logging
    request_log: Vec<String>,
    request_count: usize,

    // logging
    recv_msg_log: Vec<Lib3hServerProtocol>,

    // datastores per dna
    chain_store_list: HashMap<Address, ChainStore>,
    tracked_dna_list: HashSet<Address>,

    pub current_dna: Option<Address>,

    pub logger: TweetProxy,

    is_network_ready: bool,
    pub p2p_binding: url::Url,
}

/// Query logs
impl TestNode {
    /// Return number of Lib3hServerProtocol message this node has received
    pub fn count_recv_json_messages(&self) -> usize {
        self.recv_msg_log.len()
    }

    /// Return the ith message that this node has received and fullfills predicate
    pub fn find_recv_lib3h_msg(
        &self,
        ith: usize,
        predicate: Box<dyn Fn(&Lib3hServerProtocol) -> bool>,
    ) -> Option<Lib3hServerProtocol> {
        let mut count = 0;
        for msg in self.recv_msg_log.clone() {
            if predicate(&msg) {
                if count == ith {
                    return Some(msg);
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
        let track_dna_msg = lib3h_protocol::data_types::SpaceData {
            request_id: snowflake::ProcessUniqueId::new().to_string(),
            space_address: dna_address.clone(),
            agent_id: agent_id,
        };
        let protocol_msg = Lib3hClientProtocol::JoinSpace(track_dna_msg);
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
        let protocol_msg: Lib3hClientProtocol = {
            let track_dna_msg = SpaceData {
                request_id: "untrack_dna_req".into(),
                space_address: dna_address.clone(),
                agent_id: agent_id.clone(),
            };
            Lib3hClientProtocol::LeaveSpace(track_dna_msg)
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
                aspect: aspect_content.into(),
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
                space_address: current_dna,
                provider_agent_id: self.agent_id.clone(),
                entry: entry.clone(),
            };
            return self.send(Lib3hClientProtocol::PublishEntry(msg_data));
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
            space_address: current_dna,
            entry_address,
            request_id: self.generate_request_id(),
            requester_agent_id: self.agent_id.clone(),
            query: vec![].into(), // empty means give me the EntryData,
        };
        self.send(Lib3hClientProtocol::QueryEntry(query_data.clone()))
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
                space_address: query.space_address.clone(),
                request_id: query.request_id.clone(),
                to_agent_id: query.requester_agent_id.clone(),
                result_info: "Unknown query request".as_bytes().into(),
            };
            self.send(Lib3hClientProtocol::FailureResult(msg_data.clone()))
                .expect("Sending FailureResult failed");
            return Err(msg_data);
        }
        // Convert query to fetch
        let fetch = FetchEntryData {
            space_address: query.space_address.clone(),
            request_id: query.request_id.clone(),
            provider_agent_id: query.requester_agent_id.clone(),
            entry_address: query.entry_address.clone(),
            aspect_address_list: None,
        };
        // HandleFetchEntry
        let fetch_res = self.reply_to_HandleFetchEntry_inner(&fetch);
        if let Err(res) = fetch_res {
            self.send(Lib3hClientProtocol::FailureResult(res.clone()))
                .expect("Sending FailureResult failed");
            return Err(res);
        }
        // Convert query to fetch
        let query_res = QueryEntryResultData {
            space_address: query.space_address.clone(),
            entry_address: query.entry_address.clone(),
            request_id: query.request_id.clone(),
            requester_agent_id: query.requester_agent_id.clone(),
            responder_agent_id: self.agent_id.clone(),
            query_result: bincode::serialize(&fetch_res.unwrap().entry)
                .unwrap()
                .into(),
        };
        self.send(Lib3hClientProtocol::HandleQueryEntryResult(
            query_res.clone(),
        ))
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
            Err(res) => Lib3hClientProtocol::FailureResult(res),
            Ok(fetch) => Lib3hClientProtocol::HandleFetchEntryResult(fetch),
        };
        self.send(json_msg).expect("Sending failed");
        fetch_res
    }

    /// Node asks for some entry on the network.
    fn reply_to_HandleFetchEntry_inner(
        &mut self,
        fetch: &FetchEntryData,
    ) -> Result<FetchEntryResultData, GenericResultData> {
        // Must be tracking DNA
        if !self.is_tracking(&fetch.space_address) {
            let msg_data = GenericResultData {
                space_address: fetch.space_address.clone(),
                request_id: fetch.request_id.clone(),
                to_agent_id: fetch.provider_agent_id.clone(),
                result_info: "DNA is not tracked".as_bytes().into(),
            };
            return Err(msg_data);
        }
        // Get Entry
        let maybe_store = self.chain_store_list.get(&fetch.space_address);
        let maybe_entry = match maybe_store {
            None => {
                println!(
                    "did not find chain store for space address: {}",
                    fetch.space_address
                );
                None
            }
            Some(chain_store) => {
                println!(
                    "found chain store for space address: {}",
                    fetch.space_address
                );
                chain_store.get_entry(&fetch.entry_address)
            }
        };
        // No entry, send failure
        if maybe_entry.is_none() {
            let msg_data = GenericResultData {
                space_address: fetch.space_address.clone(),
                request_id: fetch.request_id.clone(),
                to_agent_id: fetch.provider_agent_id.clone(),
                result_info: "No entry found".as_bytes().into(),
            };
            return Err(msg_data);
        }
        // Send EntryData as binary
        let fetch_result_data = FetchEntryResultData {
            space_address: fetch.space_address.clone(),
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
        let from_agent_id = self.agent_id.clone();

        let msg_data = DirectMessageData {
            space_address: dna_address,
            request_id: request_id.clone(),
            to_agent_id: to_agent_id.clone(),
            from_agent_id,
            content: content.into(),
        };
        self.send(Lib3hClientProtocol::SendDirectMessage(msg_data))
            .expect("Sending SendMessage failed");
        request_id
    }

    /// Node sends Message on the network.
    pub fn send_response_json(&mut self, msg: DirectMessageData, response_content: Vec<u8>) {
        assert!(self.current_dna.is_some());
        let current_dna = self.current_dna.clone().unwrap();
        assert_eq!(msg.space_address, current_dna.clone());
        assert_eq!(msg.to_agent_id, self.agent_id);
        let response = DirectMessageData {
            space_address: msg.space_address.clone(),
            request_id: msg.request_id,
            to_agent_id: msg.from_agent_id.clone(),
            from_agent_id: msg.to_agent_id.clone(),
            content: response_content.into(),
        };
        self.send(Lib3hClientProtocol::HandleSendDirectMessageResult(
            response.clone(),
        ))
        .expect("Sending HandleSendMessageResult failed");
    }

    /// Node sends Message on the network.
    pub fn send_response_lib3h(&mut self, msg: DirectMessageData, response_content: Vec<u8>) {
        assert!(self.current_dna.is_some());
        let current_dna = self.current_dna.clone().unwrap();
        assert_eq!(
            String::from(msg.space_address),
            current_dna.clone().to_string()
        );
        assert_eq!(
            String::from(msg.to_agent_id),
            self.agent_id.clone().to_string()
        );
        let response = DirectMessageData {
            space_address: current_dna.clone().to_string().into(),
            request_id: msg.request_id,
            to_agent_id: msg.from_agent_id.clone(),
            from_agent_id: self.agent_id.to_string().into(),
            content: response_content,
        };
        self.send(Lib3hClientProtocol::HandleSendDirectMessageResult(
            response.clone(),
        ))
        .expect("Sending HandleSendMessageResult failed");
    }
}

/// Reply LISTS
impl TestNode {
    /// Reply to a HandleGetAuthoringEntryList request
    pub fn reply_to_HandleGetAuthoringEntryList(&mut self, request: &GetListData) -> NetResult<()> {
        assert!(self.current_dna.is_some());
        let current_dna = self.current_dna.clone().unwrap();
        assert_eq!(request.space_address, current_dna);
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
                space_address: request.space_address.clone(),
                address_map: entry_address_list,
                provider_agent_id: self.agent_id.clone(),
            };
        }
        self.send(Lib3hClientProtocol::HandleGetAuthoringEntryListResult(msg))
    }
    /// Look for the first HandleGetAuthoringEntryList request received from network module and reply
    pub fn reply_to_first_HandleGetAuthoringEntryList(&mut self) {
        let request = self
            .find_recv_lib3h_msg(
                0,
                Box::new(one_is!(Lib3hServerProtocol::HandleGetAuthoringEntryList(_))),
            )
            .expect("Did not receive any HandleGetAuthoringEntryList request");
        let get_entry_list_data =
            unwrap_to!(request => Lib3hServerProtocol::HandleGetAuthoringEntryList);
        self.reply_to_HandleGetAuthoringEntryList(&get_entry_list_data)
            .expect("Reply to HandleGetAuthoringEntryList failed.");
    }

    /// Reply to a HandleGetHoldingEntryList request
    pub fn reply_to_HandleGetHoldingEntryList(&mut self, request: &GetListData) -> NetResult<()> {
        assert!(self.current_dna.is_some());
        let current_dna = self.current_dna.clone().unwrap();
        assert_eq!(request.space_address, current_dna);
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
                space_address: request.space_address.clone(),
                address_map: entry_address_list,
                provider_agent_id: self.agent_id.clone(),
            };
        }
        self.send(Lib3hClientProtocol::HandleGetGossipingEntryListResult(msg))
    }
    /// Look for the first HandleGetHoldingEntryList request received from network module and reply
    pub fn reply_to_first_HandleGetHoldingEntryList(&mut self) {
        let request = self
            .find_recv_lib3h_msg(
                0,
                Box::new(one_is!(Lib3hServerProtocol::HandleGetGossipingEntryList(_))),
            )
            .expect("Did not receive a HandleGetHoldingEntryList request");
        // extract request data
        let get_list_data = unwrap_to!(request => Lib3hServerProtocol::HandleGetGossipingEntryList);
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
        let (sender, receiver) = unbounded::<Lib3hServerProtocol>();
        // create a new P2pNetwork instance with the handler that will send the received Protocol to a channel
        let agent_id = agent_id_arg.clone();
        let p2p_connection = P2pNetwork::new(
            NetHandler::new(Box::new(move |r| {
                log_tt!("p2pnode", "<<< ({}) handler: {:?}", agent_id_arg, r);
                sender.send(r?)?;
                Ok(())
            })),
            config.clone(),
        )
        .expect("Failed to create P2pNetwork");

        let p2p_binding = p2p_connection.p2p_endpoint();
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
            p2p_binding,
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
        bootstrap_nodes: Vec<url::Url>,
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
    pub fn try_recv(&mut self) -> NetResult<Lib3hServerProtocol> {
        let data = self.receiver.try_recv()?;

        self.recv_msg_log.push(data.clone());
        Ok(data)
    }

    /// See if there is a message to receive, and log it
    /// See if there is a message to receive, and log it
    /// return a Lib3hServerProtocol if the received message is of that type
    #[cfg_attr(tarpaulin, skip)]
    pub fn try_recv_lib3h(&mut self) -> NetResult<Lib3hServerProtocol> {
        let data = self.receiver.try_recv()?;

        self.recv_msg_log.push(data.clone());

        match data {
            Lib3hServerProtocol::P2pReady => {
                let dbg_msg = format!("<< ({}) recv ** P2pReady **", self.agent_id);
                self.logger.d(&dbg_msg);
                self.is_network_ready = true;
                bail!("received P2pReady");
            }
            Lib3hServerProtocol::Terminated => {
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

        self.handle_lib3h(data.clone());
        Ok(data)
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
        let maybe_request =
            self.wait_lib3h(Box::new(one_is!(Lib3hServerProtocol::HandleFetchEntry(_))));
        if maybe_request.is_none() {
            return false;
        }
        let request = maybe_request.unwrap();
        // extract msg data
        let fetch_data = unwrap_to!(request => Lib3hServerProtocol::HandleFetchEntry);
        // Respond
        self.reply_to_HandleFetchEntry(&fetch_data)
            .expect("Reply to HandleFetchEntry should work");
        true
    }

    /// wait to receive a HandleFetchEntry request and automatically reply
    /// return true if a HandleFetchEntry has been received
    pub fn wait_HandleQueryEntry_and_reply(&mut self) -> bool {
        let maybe_request =
            self.wait_lib3h(Box::new(one_is!(Lib3hServerProtocol::HandleQueryEntry(_))));
        if maybe_request.is_none() {
            return false;
        }
        let request = maybe_request.unwrap();
        // extract msg data
        let query_data = unwrap_to!(request => Lib3hServerProtocol::HandleQueryEntry);
        // Respond
        self.reply_to_HandleQueryEntry(&query_data)
            .expect("Reply to HandleFetchEntry should work");
        true
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

            if let Ok(lib3h_msg) = self.try_recv() {
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

    /// handle all types of lib3h server messages
    #[cfg_attr(tarpaulin, skip)]
    fn handle_lib3h(&mut self, msg: Lib3hServerProtocol) {
        match msg {
            Lib3hServerProtocol::Terminated => {
                // FIXME
            }
            Lib3hServerProtocol::P2pReady => {
                // FIXME
            }
            Lib3hServerProtocol::SuccessResult(_msg) => {
                // n/a
            }
            Lib3hServerProtocol::FailureResult(_msg) => {
                // n/a
            }
            Lib3hServerProtocol::Connected(_) => {
                // n/a
            }
            Lib3hServerProtocol::SendDirectMessageResult(_) => {
                // n/a
            }
            Lib3hServerProtocol::HandleSendDirectMessage(_msg) => {
                // log the direct message sent to us
                // FIXME
            }
            Lib3hServerProtocol::HandleFetchEntry(_) => {
                // n/a
            }

            Lib3hServerProtocol::HandleStoreEntryAspect(msg) => {
                if self.is_tracking(&msg.space_address) {
                    // Store data in local datastore
                    let chain_store = self
                        .chain_store_list
                        .get_mut(&msg.space_address)
                        .expect("No dna_store for this DNA");
                    let res = chain_store.hold_aspect(&msg.entry_address, &msg.entry_aspect);
                    self.logger.d(&format!(
                        "({}) auto-store of aspect: {} - {} -> {}",
                        self.agent_id,
                        msg.entry_address,
                        msg.entry_aspect.aspect_address,
                        res.is_ok()
                    ));
                } else {
                    panic!("not tracking this space address: {:?}", msg);
                }
            }

            Lib3hServerProtocol::QueryEntryResult(_msg) => {
                // n/a
            }
            Lib3hServerProtocol::HandleQueryEntry(_msg) => {
                // n/a
            }
            // -- Publish & Hold data -- //
            Lib3hServerProtocol::HandleGetAuthoringEntryList(_) => {
                // n/a
            }
            Lib3hServerProtocol::HandleGetGossipingEntryList(_) => {
                // n/a
            }
            msg => self
                .logger
                .d(&format!("unhandled protocol message: {:?}", msg)),
        }
    }
}

impl NetSend for TestNode {
    /// send a Protocol message to the p2p network instance
    fn send(&mut self, data: Lib3hClientProtocol) -> NetResult<()> {
        self.logger
            .d(&format!(">> ({}) send: {:?}", self.agent_id, data));
        self.p2p_connection.send(data)
    }
}
