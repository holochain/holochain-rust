#![allow(non_snake_case)]

use holochain_net::{
    connection::{
        json_protocol::{
            DhtMetaData, EntryData, EntryListData, FailureResultData, FetchEntryData,
            FetchEntryResultData, FetchMetaData, FetchMetaResultData, GetListData, JsonProtocol,
            MessageData, MetaKey, MetaListData, TrackDnaData,
        },
        net_connection::NetSend,
        protocol::Protocol,
        NetResult,
    },
    p2p_config::*,
    p2p_network::P2pNetwork,
    tweetlog::{TweetProxy, *},
};
use holochain_persistence_api::cas::content::Address;

use super::{
    create_config::{create_ipc_config, create_lib3h_config},
    dna_store::DnaStore,
};
use crossbeam_channel::{unbounded, Receiver};
use holochain_net::connection::net_connection::NetHandler;
use lib3h_protocol::{
    data_types::DirectMessageData, protocol_client::Lib3hClientProtocol,
    protocol_server::Lib3hServerProtocol,
};
use std::{
    collections::{HashMap, HashSet},
    convert::TryFrom,
};

static TIMEOUT_MS: usize = 5000;

/// Core Mock
pub struct TestNode {
    // Need to hold the tempdir to keep it alive, otherwise we will get a dir error.
    _maybe_temp_dir: Option<tempfile::TempDir>,
    p2p_connection: P2pNetwork,
    receiver: Receiver<Protocol>,
    pub config: P2pConfig,

    pub agent_id: String,

    // my request logging
    request_log: Vec<String>,
    request_count: usize,

    // logging
    recv_msg_log: Vec<Protocol>,

    // datastores per dna
    dna_stores: HashMap<Address, DnaStore>,
    tracked_dnas: HashSet<Address>,

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
        if self.tracked_dnas.contains(dna_address) {
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
            self.tracked_dnas.insert(dna_address.clone());
            if !self.dna_stores.contains_key(dna_address) {
                self.dna_stores
                    .insert(dna_address.clone(), DnaStore::new(dna_address.clone()));
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
        if !self.tracked_dnas.contains(dna_address) {
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
            self.tracked_dnas.remove(dna_address);
        }
        res
    }

    ///
    pub fn is_tracking(&self, dna_address: &Address) -> bool {
        self.tracked_dnas.contains(dna_address)
    }

    ///
    pub fn set_current_dna(&mut self, dna_address: &Address) {
        if self.dna_stores.contains_key(dna_address) {
            self.current_dna = Some(dna_address.clone());
        };
    }
}

/// publish, hold
impl TestNode {
    pub fn author_entry(
        &mut self,
        entry_address: &Address,
        entry_content: &serde_json::Value,
        can_publish: bool,
    ) -> NetResult<()> {
        assert!(self.current_dna.is_some());
        let current_dna = self.current_dna.clone().unwrap();
        {
            let dna_store = self
                .dna_stores
                .get_mut(&current_dna)
                .expect("No dna_store for this DNA");
            assert!(!dna_store.authored_entry_store.get(&entry_address).is_some());
            assert!(!dna_store.entry_store.get(&entry_address).is_some());
            dna_store
                .authored_entry_store
                .insert(entry_address.clone(), entry_content.clone());
        }
        if can_publish {
            let msg_data = EntryData {
                dna_address: current_dna,
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
        assert!(self.current_dna.is_some());
        let current_dna = self.current_dna.clone().unwrap();
        let meta_key = (entry_address.clone(), attribute.to_string());

        // bookkeep
        {
            let dna_store = self
                .dna_stores
                .get_mut(&current_dna)
                .expect("No dna_store for this DNA");
            // Must not already have meta
            assert!(!dna_store
                .meta_store
                .has(meta_key.clone(), link_entry_address));
            dna_store
                .authored_meta_store
                .insert(meta_key.clone(), link_entry_address.clone());
        }
        // publish it
        if can_publish {
            let msg_data = DhtMetaData {
                dna_address: self.current_dna.clone().unwrap(),
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
        assert!(self.current_dna.is_some());
        let current_dna = self.current_dna.clone().unwrap();
        let dna_store = self
            .dna_stores
            .get_mut(&current_dna)
            .expect("No dna_store for this DNA");
        assert!(!dna_store.authored_entry_store.get(&entry_address).is_some());
        assert!(!dna_store.entry_store.get(&entry_address).is_some());
        dna_store
            .entry_store
            .insert(entry_address.clone(), entry_content.clone());
    }

    pub fn hold_meta(
        &mut self,
        entry_address: &Address,
        attribute: &str,
        link_entry_address: &serde_json::Value,
    ) {
        assert!(self.current_dna.is_some());
        let current_dna = self.current_dna.clone().unwrap();
        let dna_store = self
            .dna_stores
            .get_mut(&current_dna)
            .expect("No dna_store for this DNA");
        let meta_key = (entry_address.clone(), attribute.to_string());
        // Must not already have meta
        assert!(!dna_store
            .authored_meta_store
            .has(meta_key.clone(), link_entry_address));
        dna_store
            .meta_store
            .insert(meta_key, link_entry_address.clone());
    }
}

/// fetch & sendMessage
impl TestNode {
    /// generate a new request_id
    fn generate_request_id(&mut self) -> String {
        self.request_count += 1;
        let request_id = format!("req_{}_{}", self.agent_id, self.request_count);
        self.request_log.push(request_id.clone());
        request_id
    }

    /// Node asks for some entry on the network.
    pub fn request_entry(&mut self, entry_address: Address) -> FetchEntryData {
        assert!(self.current_dna.is_some());
        let current_dna = self.current_dna.clone().unwrap();
        let fetch_data = FetchEntryData {
            request_id: self.generate_request_id(),
            dna_address: current_dna,
            requester_agent_id: self.agent_id.clone(),
            entry_address,
        };
        self.send(JsonProtocol::FetchEntry(fetch_data.clone()).into())
            .expect("Sending FetchEntry failed");
        fetch_data
    }

    /// Node asks for some meta on the network.
    pub fn request_meta(&mut self, entry_address: Address, attribute: String) -> FetchMetaData {
        assert!(self.current_dna.is_some());
        let current_dna = self.current_dna.clone().unwrap();
        let fetch_meta = FetchMetaData {
            request_id: self.generate_request_id(),
            dna_address: current_dna,
            requester_agent_id: self.agent_id.to_string(),
            entry_address,
            attribute,
        };
        self.send(JsonProtocol::FetchMeta(fetch_meta.clone()).into())
            .expect("Sending FetchMeta failed");
        fetch_meta
    }

    /// Node sends Message on the network.
    pub fn send_message(&mut self, to_agent_id: String, content: serde_json::Value) -> String {
        println!("set_current_dna: {:?}", self.current_dna);
        assert!(self.current_dna.is_some());
        let dna_address = self.current_dna.clone().unwrap();
        let request_id = self.generate_request_id();
        let from_agent_id = self.agent_id.to_string();

        let p = if self.is_json {
            let msg_data = MessageData {
                dna_address,
                from_agent_id,
                request_id: request_id.clone(),
                to_agent_id,
                content,
            };
            JsonProtocol::SendMessage(msg_data.clone()).into()
        } else {
            let msg_data = DirectMessageData {
                space_address: dna_address.to_string().into_bytes(),
                request_id: request_id.clone(),
                to_agent_id: to_agent_id.to_string().into_bytes(),
                from_agent_id: from_agent_id.to_string().into_bytes(),
                content: content.to_string().into_bytes(),
            };
            Lib3hClientProtocol::SendDirectMessage(msg_data.clone()).into()
        };
        self.send(p).expect("Sending SendMessage failed");
        request_id
    }

    /// Node sends Message on the network.
    pub fn send_reponse_json(&mut self, msg: MessageData, response_content: serde_json::Value) {
        assert!(self.current_dna.is_some());
        let current_dna = self.current_dna.clone().unwrap();
        assert_eq!(msg.dna_address, current_dna.clone());
        assert_eq!(msg.to_agent_id, self.agent_id);
        let response = MessageData {
            dna_address: current_dna.clone(),
            from_agent_id: self.agent_id.to_string(),
            request_id: msg.request_id,
            to_agent_id: msg.from_agent_id.clone(),
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

// Replies
impl TestNode {
    // -- FETCH -- //

    /// Send a reponse to a FetchDhtData request
    pub fn reply_to_HandleFetchEntry(&mut self, request: &FetchEntryData) -> NetResult<()> {
        assert!(self.current_dna.is_some());
        let current_dna = self.current_dna.clone().unwrap();
        assert_eq!(request.dna_address, current_dna);
        let msg;
        {
            // Get data from local datastores
            let dna_store = self
                .dna_stores
                .get_mut(&current_dna)
                .expect("No dna_store for this DNA");
            let mut maybe_data = dna_store.authored_entry_store.get(&request.entry_address);
            if maybe_data.is_none() {
                maybe_data = dna_store.entry_store.get(&request.entry_address);
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
        self.logger.t(&format!(
            "> ({}) reply_to_HandleFetchEntry() sending",
            self.agent_id,
        ));
        self.send(msg)
    }

    /// Send a reponse to a FetchDhtMetaData request
    pub fn reply_to_HandleFetchMeta(&mut self, request: &FetchMetaData) -> NetResult<()> {
        assert!(self.current_dna.is_some());
        let current_dna = self.current_dna.clone().unwrap();
        assert_eq!(request.dna_address, current_dna);
        let msg;
        {
            let dna_store = self
                .dna_stores
                .get_mut(&current_dna)
                .expect("No dna_store for this DNA");
            // Get meta from local datastores
            let meta_key = (request.entry_address.clone(), request.attribute.clone());
            let mut metas = dna_store.authored_meta_store.get(meta_key.clone());
            if metas.is_empty() {
                metas = dna_store.meta_store.get(meta_key);
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
        assert!(self.current_dna.is_some());
        let current_dna = self.current_dna.clone().unwrap();
        assert_eq!(request.dna_address, current_dna);
        let msg;
        {
            let dna_store = self
                .dna_stores
                .get_mut(&current_dna)
                .expect("No dna_store for this DNA");
            let entry_address_list = dna_store
                .authored_entry_store
                .iter()
                .map(|(k, _)| k.clone())
                .collect();
            msg = EntryListData {
                entry_address_list: entry_address_list,
                request_id: request.request_id.clone(),
                dna_address: request.dna_address.clone(),
            };
        }
        self.send(JsonProtocol::HandleGetPublishingEntryListResult(msg).into())
    }
    /// Look for the first HandleGetPublishingEntryList request received from network module and reply
    pub fn reply_to_first_HandleGetPublishingEntryList(&mut self) {
        let request = self
            .find_recv_json_msg(
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
        assert!(self.current_dna.is_some());
        let current_dna = self.current_dna.clone().unwrap();
        assert_eq!(request.dna_address, current_dna);
        let msg;
        {
            let dna_store = self
                .dna_stores
                .get_mut(&current_dna)
                .expect("No dna_store for this DNA");
            msg = MetaListData {
                request_id: request.request_id.clone(),
                dna_address: request.dna_address.clone(),
                meta_list: dna_store.authored_meta_store.get_all(),
            };
        }
        self.send(JsonProtocol::HandleGetPublishingMetaListResult(msg).into())
    }
    /// Look for the first HandleGetPublishingMetaList request received from network module and reply
    pub fn reply_to_first_HandleGetPublishingMetaList(&mut self) {
        self.logger.t(&format!(
            "--- HandleGetPublishingMetaList: {}",
            self.agent_id
        ));
        let request = self
            .find_recv_json_msg(
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
        assert!(self.current_dna.is_some());
        let current_dna = self.current_dna.clone().unwrap();
        assert_eq!(request.dna_address, current_dna);
        let msg;
        {
            let dna_store = self
                .dna_stores
                .get_mut(&current_dna)
                .expect("No dna_store for this DNA");
            let entry_address_list = dna_store
                .entry_store
                .iter()
                .map(|(k, _)| k.clone())
                .collect();
            msg = EntryListData {
                request_id: request.request_id.clone(),
                dna_address: request.dna_address.clone(),
                entry_address_list: entry_address_list,
            };
        }
        self.send(JsonProtocol::HandleGetHoldingEntryListResult(msg).into())
    }
    /// Look for the first HandleGetHoldingEntryList request received from network module and reply
    pub fn reply_to_first_HandleGetHoldingEntryList(&mut self) {
        let request = self
            .find_recv_json_msg(
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
        assert!(self.current_dna.is_some());
        let current_dna = self.current_dna.clone().unwrap();
        assert_eq!(request.dna_address, current_dna);
        let msg;
        {
            let dna_store = self
                .dna_stores
                .get_mut(&current_dna)
                .expect("No dna_store for this DNA");
            msg = MetaListData {
                request_id: request.request_id.clone(),
                dna_address: request.dna_address.clone(),
                meta_list: dna_store.meta_store.get_all(),
            };
        }
        self.send(JsonProtocol::HandleGetHoldingMetaListResult(msg).into())
    }
    /// Look for the first HandleGetHoldingMetaList request received from network module and reply
    pub fn reply_to_first_HandleGetHoldingMetaList(&mut self) {
        let request = self
            .find_recv_json_msg(
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

impl TestNode {
    /// Private constructor
    #[cfg_attr(tarpaulin, skip)]
    pub fn new_with_config(
        agent_id_arg: String,
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
            dna_stores: HashMap::new(),
            tracked_dnas: HashSet::new(),
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
    pub fn new_with_unique_memory_network(agent_id: String) -> Self {
        let config = P2pConfig::new_with_unique_memory_backend();
        return TestNode::new_with_config(agent_id, &config, None);
    }

    /// Constructor for an IPC node that uses an existing n3h process and a temp folder
    #[cfg_attr(tarpaulin, skip)]
    pub fn new_with_uri_ipc_network(agent_id: String, ipc_binding: &str) -> Self {
        let p2p_config = P2pConfig::default_ipc_uri(Some(ipc_binding));
        return TestNode::new_with_config(agent_id, &p2p_config, None);
    }

    /// Constructor for an IPC node that uses an existing n3h process and a temp folder
    #[cfg_attr(tarpaulin, skip)]
    pub fn new_with_lib3h(
        agent_id: String,
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
        agent_id: String,
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
    #[allow(non_snake_case)]
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

    /// wait to receive a HandleFetchMeta request and automatically reply
    /// return true if a HandleFetchMeta has been received
    #[allow(non_snake_case)]
    pub fn wait_HandleFetchMeta_and_reply(&mut self) -> bool {
        let maybe_request = self.wait_json(Box::new(one_is!(JsonProtocol::HandleFetchMeta(_))));
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
            JsonProtocol::GetStateResult(state) => {
                if !state.bindings.is_empty() {
                    self.p2p_binding = state.bindings[0].clone();
                }
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
            JsonProtocol::HandleSendMessage(_msg) => {
                // log the direct message sent to us
                // FIXME
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
            JsonProtocol::HandleFetchEntry(_msg) => {
                // n/a
            }
            JsonProtocol::HandleFetchEntryResult(_msg) => {
                // n/a
            }

            JsonProtocol::PublishEntry(_msg) => {
                panic!("Core should not receive PublishDhtData message");
            }
            JsonProtocol::HandleStoreEntry(msg) => {
                if self.is_tracking(&msg.dna_address) {
                    // Store data in local datastore
                    let mut dna_store = self
                        .dna_stores
                        .get_mut(&msg.dna_address)
                        .expect("No dna_store for this DNA");
                    dna_store
                        .entry_store
                        .insert(msg.entry_address, msg.entry_content);
                }
            }
            JsonProtocol::HandleDropEntry(msg) => {
                if self.is_tracking(&msg.dna_address) {
                    // Remove data in local datastore
                    let mut dna_store = self
                        .dna_stores
                        .get_mut(&msg.dna_address)
                        .expect("No dna_store for this DNA");
                    dna_store.entry_store.remove(&msg.entry_address);
                }
            }

            JsonProtocol::FetchMeta(_msg) => {
                panic!("Core should not receive FetchDhtMeta message");
            }
            JsonProtocol::FetchMetaResult(_msg) => {
                // n/a
            }
            JsonProtocol::HandleFetchMeta(_msg) => {
                // n/a
            }
            JsonProtocol::HandleFetchMetaResult(_msg) => {
                // n/a
            }

            JsonProtocol::PublishMeta(_msg) => {
                panic!("Core should not receive PublishDhtMeta message");
            }
            JsonProtocol::HandleStoreMeta(msg) => {
                if self.is_tracking(&msg.dna_address) {
                    // Store data in local datastore
                    let meta_key = (msg.entry_address, msg.attribute);
                    let mut dna_store = self
                        .dna_stores
                        .get_mut(&msg.dna_address)
                        .expect("No dna_store for this DNA");
                    for content in msg.content_list {
                        dna_store.meta_store.insert(meta_key.clone(), content);
                    }
                }
            }
            // TODO
            //            JsonProtocol::HandleDropMeta(msg) => {
            //                assert!(self.is_tracking(&msg.dna_address));
            //                // Remove data in local datastore
            //                self.meta_store.remove(&(msg.entry_address, msg.attribute));
            //            }

            // -- Publish & Hold data -- //
            JsonProtocol::HandleGetPublishingEntryList(_) => {
                // n/a
            }
            JsonProtocol::HandleGetPublishingEntryListResult(_) => {
                panic!("Core should not receive HandleGetPublishingDataListResult message");
            }
            JsonProtocol::HandleGetHoldingEntryList(_) => {
                // n/a
            }
            // Our request for the hold_list has returned
            JsonProtocol::HandleGetHoldingEntryListResult(_) => {
                panic!("Core should not receive HandleGetHoldingDataListResult message");
            }

            // -- Publish & Hold meta -- //
            JsonProtocol::HandleGetPublishingMetaList(_) => {
                // n/a
            }
            JsonProtocol::HandleGetPublishingMetaListResult(_) => {
                panic!("Core should not receive HandleGetPublishingMetaListResult message");
            }
            JsonProtocol::HandleGetHoldingMetaList(_) => {
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

impl NetSend for TestNode {
    /// send a Protocol message to the p2p network instance
    fn send(&mut self, data: Protocol) -> NetResult<()> {
        self.logger
            .d(&format!(">> ({}) send: {:?}", self.agent_id, data));
        self.p2p_connection.send(data)
    }
}
