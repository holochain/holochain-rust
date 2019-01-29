use holochain_net::{p2p_config::*, p2p_network::P2pNetwork};
use holochain_net_connection::{
    json_protocol::{
        DhtMetaData, EntryData, EntryListData, FailureResultData, FetchEntryData,
        FetchEntryResultData, FetchMetaData, FetchMetaResultData, GetListData, JsonProtocol,
        MessageData, MetaListData,
    },
    net_connection::NetSend,
    protocol::Protocol,
    NetResult,
};
use std::{collections::HashMap, convert::TryFrom, sync::mpsc};

use holochain_core_types::cas::content::Address;

static TIMEOUT_MS: usize = 5000;

/// Core Mock
pub struct P2pNode {
    // Need to hold the tempdir to keep it alive, otherwise we will get a dir error.
    _maybe_temp_dir: Option<tempfile::TempDir>,
    p2p_connection: P2pNetwork,
    receiver: mpsc::Receiver<Protocol>,
    pub config: P2pConfig,

    pub agent_id: String,

    // logging
    recv_msg_log: Vec<Protocol>,
    recv_dm_log: Vec<MessageData>,

    // datastores
    // TODO: Have a datastore per dna ; perhaps in a CoreMock struct
    pub entry_store: HashMap<Address, serde_json::Value>,
    pub meta_store: HashMap<(Address, String), serde_json::Value>,
    pub authored_entry_store: HashMap<Address, serde_json::Value>,
    pub authored_meta_store: HashMap<(Address, String), serde_json::Value>,
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

// Publish & Hold
impl P2pNode {
    pub fn author_entry(
        &mut self,
        dna_address: &Address,
        data_address: &Address,
        data_content: &serde_json::Value,
        can_publish: bool,
    ) -> NetResult<()> {
        assert!(!self.authored_entry_store.get(&data_address).is_some());
        assert!(!self.entry_store.get(&data_address).is_some());
        self.authored_entry_store
            .insert(data_address.clone(), data_content.clone());
        if can_publish {
            let msg_data = EntryData {
                dna_address: dna_address.clone(),
                provider_agent_id: self.agent_id.clone(),
                entry_address: data_address.clone(),
                entry_content: data_content.clone(),
            };
            return self.send(JsonProtocol::PublishEntry(msg_data).into());
        }
        // Done
        Ok(())
    }

    pub fn author_meta(
        &mut self,
        dna_address: &Address,
        data_address: &Address,
        attribute: &str,
        content: &serde_json::Value,
        can_publish: bool,
    ) -> NetResult<()> {
        let meta_address = (data_address.clone(), attribute.to_string());
        assert!(!self.authored_meta_store.get(&meta_address).is_some());
        assert!(!self.meta_store.get(&meta_address).is_some());
        self.authored_meta_store.insert(
            (data_address.clone(), attribute.to_string()),
            content.clone(),
        );
        if can_publish {
            let msg_data = DhtMetaData {
                dna_address: dna_address.clone(),
                provider_agent_id: self.agent_id.clone(),
                entry_address: data_address.clone(),
                attribute: attribute.to_string(),
                content: content.clone(),
            };
            return self.send(JsonProtocol::PublishMeta(msg_data).into());
        }
        // Done
        Ok(())
    }

    pub fn hold_data(&mut self, data_address: &Address, data_content: &serde_json::Value) {
        assert!(!self.authored_entry_store.get(&data_address).is_some());
        assert!(!self.entry_store.get(&data_address).is_some());
        self.entry_store
            .insert(data_address.clone(), data_content.clone());
    }

    pub fn hold_meta(
        &mut self,
        data_address: &Address,
        attribute: &str,
        content: &serde_json::Value,
    ) {
        let meta_address = (data_address.clone(), attribute.to_string());
        assert!(!self.authored_meta_store.get(&meta_address).is_some());
        assert!(!self.meta_store.get(&meta_address).is_some());
        self.meta_store.insert(
            (data_address.clone(), attribute.to_string()),
            content.clone(),
        );
    }
}

// Replies
impl P2pNode {
    // -- FETCH -- //

    /// Send a reponse to a FetchDhtData request
    pub fn reply_fetch_data(&mut self, request: &FetchEntryData) -> NetResult<()> {
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
    pub fn reply_fetch_meta(&mut self, request: &FetchMetaData) -> NetResult<()> {
        let msg;
        {
            // Get meta from local datastores
            let meta_pair = &(request.entry_address.clone(), request.attribute.clone());
            let mut maybe_data = self.authored_meta_store.get(meta_pair);
            if maybe_data.is_none() {
                maybe_data = self.meta_store.get(meta_pair);
            }
            // if meta not found send empty content (will make the aggregation easier)
            let data = match maybe_data.clone() {
                Some(data) => data.clone(),
                None => json!(""),
            };
            msg = FetchMetaResultData {
                request_id: request.request_id.clone(),
                requester_agent_id: request.requester_agent_id.clone(),
                dna_address: request.dna_address.clone(),
                provider_agent_id: self.agent_id.clone(),
                entry_address: request.entry_address.clone(),
                attribute: request.attribute.clone(),
                content: data.clone(),
            };
        }
        self.send(JsonProtocol::HandleFetchMetaResult(msg).into())
    }

    // -- LISTS -- //

    pub fn reply_get_publish_data_list(&mut self, request: &GetListData) -> NetResult<()> {
        let data_address_list = self
            .authored_entry_store
            .iter()
            .map(|(k, _)| k.clone())
            .collect();
        let msg = EntryListData {
            entry_address_list: data_address_list,
            request_id: request.request_id.clone(),
            dna_address: request.dna_address.clone(),
        };
        self.send(JsonProtocol::HandleGetPublishingEntryListResult(msg).into())
    }

    pub fn reply_get_publish_meta_list(&mut self, request: &GetListData) -> NetResult<()> {
        let meta_list = self
            .authored_meta_store
            .iter()
            .map(|(k, _)| k.clone())
            .collect();
        let msg = MetaListData {
            meta_list,
            request_id: request.request_id.clone(),
            dna_address: request.dna_address.clone(),
        };
        self.send(JsonProtocol::HandleGetPublishingMetaListResult(msg).into())
    }

    pub fn reply_get_holding_data_list(&mut self, request: &GetListData) -> NetResult<()> {
        let data_address_list = self.entry_store.iter().map(|(k, _)| k.clone()).collect();
        let msg = EntryListData {
            entry_address_list: data_address_list,
            request_id: request.request_id.clone(),
            dna_address: request.dna_address.clone(),
        };
        self.send(JsonProtocol::HandleGetHoldingEntryListResult(msg).into())
    }

    pub fn reply_get_holding_meta_list(&mut self, request: &GetListData) -> NetResult<()> {
        let meta_list = self.meta_store.iter().map(|(k, _)| k.clone()).collect();
        let msg = MetaListData {
            meta_list,
            request_id: request.request_id.clone(),
            dna_address: request.dna_address.clone(),
        };
        self.send(JsonProtocol::HandleGetHoldingMetaListResult(msg).into())
    }
}

impl P2pNode {
    /// Private constructor
    #[cfg_attr(tarpaulin, skip)]
    pub fn new_with_config(
        agent_id_arg: String,
        config: &P2pConfig,
        _maybe_temp_dir: Option<tempfile::TempDir>,
    ) -> Self {
        // use a mpsc channel for messaging between p2p connection and main thread
        let (sender, receiver) = mpsc::channel::<Protocol>();
        // create a new P2pNetwork instance with the handler that will send the received Protocol to a channel

        let agent_id = agent_id_arg.clone();

        let p2p_connection = P2pNetwork::new(
            Box::new(move |r| {
                // Debugging code (do not remove)
                // println!("<<< ({}) handler: {:?}", agent_id_arg, r);
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
            recv_msg_log: Vec::new(),
            recv_dm_log: Vec::new(),
            entry_store: HashMap::new(),
            meta_store: HashMap::new(),
            authored_entry_store: HashMap::new(),
            authored_meta_store: HashMap::new(),
        }
    }

    /// Constructor for an in-memory P2P Network
    #[cfg_attr(tarpaulin, skip)]
    pub fn new_with_unique_memory_network(agent_id: String) -> Self {
        let config = P2pConfig::new_with_unique_memory_backend();
        return P2pNode::new_with_config(agent_id, &config, None);
    }

    /// Constructor for an IPC node that uses an existing n3h process and a temp folder
    #[cfg_attr(tarpaulin, skip)]
    pub fn new_with_uri_ipc_network(agent_id: String, ipc_binding: &str) -> Self {
        let p2p_config = P2pConfig::default_ipc_uri(Some(ipc_binding));
        return P2pNode::new_with_config(agent_id, &p2p_config, None);
    }

    /// Constructor for an IPC node that spawns and uses a n3h process and a temp folder
    #[cfg_attr(tarpaulin, skip)]
    pub fn new_with_spawn_ipc_network(
        agent_id: String,
        n3h_path: &str,
        maybe_config_filepath: Option<&str>,
        bootstrap_nodes: Vec<String>,
    ) -> Self {
        let (p2p_config, temp_dir) =
            create_ipc_config(n3h_path, maybe_config_filepath, bootstrap_nodes);
        return P2pNode::new_with_config(agent_id, &p2p_config, Some(temp_dir));
    }

    /// See if there is a message to receive, and log it
    /// return a JsonProtocol if the received message is of that type
    #[cfg_attr(tarpaulin, skip)]
    pub fn try_recv(&mut self) -> NetResult<JsonProtocol> {
        let data = self.receiver.try_recv()?;
        // Debugging code: Print non-ping messages
        // match data {
        //     Protocol::NamedBinary(_) => println!("<< ({}) recv: {:?}", self.agent_id, data),
        //     Protocol::Json(_) => println!("<< ({}) recv: {:?}", self.agent_id, data),
        //     _ => (),
        // };

        self.recv_msg_log.push(data.clone());

        match JsonProtocol::try_from(&data) {
            Ok(r) => {
                self.handle(r.clone());
                Ok(r)
            }
            Err(e) => {
                let s = format!("{:?}", e);
                if !s.contains("Empty") && !s.contains("Pong(PongData") {
                    println!("###### Received parse error ###### {} {:?}", s, data);
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

            if let Ok(_p2p_msg) = self.try_recv() {
                // Debugging code (do not remove)
                //                println!(
                //                    "({})::listen() - received: {:?}",
                //                    self.agent_id, p2p_msg
                //                );
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
        self.reply_fetch_data(&fetch_data)
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
        self.reply_fetch_meta(&fetch_meta)
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
                println!("({})::wait() - received: {:?}", self.agent_id, p2p_msg);
                did_something = true;
                if predicate(&p2p_msg) {
                    println!("\t ({})::wait() - match", self.agent_id);
                    return Some(p2p_msg);
                } else {
                    println!("\t ({})::wait() - NO match", self.agent_id);
                }
            }

            if !did_something {
                std::thread::sleep(std::time::Duration::from_millis(10));
                time_ms += 10;
                if time_ms > timeout_ms {
                    println!("({})::wait() has TIMEOUT", self.agent_id);
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
                self.reply_fetch_data(&msg)
                    .expect("Should reply to a HandleFetchEntry");
            }
            JsonProtocol::HandleFetchEntryResult(_msg) => {
                // n/a
            }

            JsonProtocol::PublishEntry(_msg) => {
                panic!("Core should not receive PublishDhtData message");
            }
            JsonProtocol::HandleStoreEntry(msg) => {
                // Store data in local datastore
                self.entry_store
                    .insert(msg.entry_address, msg.entry_content);
            }
            JsonProtocol::HandleDropEntry(msg) => {
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
                self.reply_fetch_meta(&msg)
                    .expect("Should reply to a HandleFetchMeta");
            }
            JsonProtocol::HandleFetchMetaResult(_msg) => {
                // n/a
            }

            JsonProtocol::PublishMeta(_msg) => {
                panic!("Core should not receive PublishDhtMeta message");
            }
            JsonProtocol::HandleStoreMeta(msg) => {
                // Store data in local datastore
                self.meta_store
                    .insert((msg.entry_address, msg.attribute), msg.content);
            }
            JsonProtocol::HandleDropMeta(msg) => {
                // Remove data in local datastore
                self.meta_store.remove(&(msg.entry_address, msg.attribute));
            }

            // -- Publish & Hold data -- //
            JsonProtocol::HandleGetPublishingEntryList(_msg) => {
                // n/a
            }
            JsonProtocol::HandleGetPublishingEntryListResult(_) => {
                panic!("Core should not receive HandleGetPublishingDataListResult message");
            }
            JsonProtocol::HandleGetHoldingEntryList(_msg) => {
                // n/a
            }
            // Our request for the hold_list has returned
            JsonProtocol::HandleGetHoldingEntryListResult(_) => {
                panic!("Core should not receive HandleGetHoldingDataListResult message");
            }

            // -- Publish & Hold meta -- //
            JsonProtocol::HandleGetPublishingMetaList(_msg) => {
                // n/a
            }
            JsonProtocol::HandleGetPublishingMetaListResult(_) => {
                panic!("Core should not receive HandleGetPublishingMetaListResult message");
            }
            JsonProtocol::HandleGetHoldingMetaList(_msg) => {
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
        // Debugging code (do not delete)
        // println!(">> ({}) send: {:?}", self.agent_id, data);
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

    println!("create_ipc_config() dir = {}\n", dir);

    // Create config
    let config = match maybe_config_filepath {
        Some(filepath) => {
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
            }})).unwrap()
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
            .unwrap()
        }
    };
    return (config, dir_ref);
}
