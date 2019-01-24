use holochain_net::{p2p_config::*, p2p_network::P2pNetwork};
use holochain_net_connection::{
    json_protocol::{
        DhtData, DhtMetaData, FailureResultData, FetchDhtData, FetchDhtMetaData, GetListData,
        HandleDhtMetaResultData, HandleDhtResultData, HandleListResultData,
        HandleMetaListResultData, JsonProtocol, MessageData,
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
    pub data_store: HashMap<Address, serde_json::Value>,
    pub meta_store: HashMap<(Address, String), serde_json::Value>,
    pub authored_data_store: HashMap<Address, serde_json::Value>,
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
    pub fn author_data(
        &mut self,
        dna_address: &Address,
        data_address: &Address,
        data_content: &serde_json::Value,
        can_publish: bool,
    ) -> NetResult<()> {
        assert!(!self.authored_data_store.get(&data_address).is_some());
        assert!(!self.data_store.get(&data_address).is_some());
        self.authored_data_store
            .insert(data_address.clone(), data_content.clone());
        if can_publish {
            let msg_data = DhtData {
                dna_address: dna_address.clone(),
                provider_agent_id: self.agent_id.clone(),
                data_address: data_address.clone(),
                data_content: data_content.clone(),
            };
            return self.send(JsonProtocol::PublishDhtData(msg_data).into());
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
                data_address: data_address.clone(),
                attribute: attribute.to_string(),
                content: content.clone(),
            };
            return self.send(JsonProtocol::PublishDhtMeta(msg_data).into());
        }
        // Done
        Ok(())
    }

    pub fn hold_data(&mut self, data_address: &Address, data_content: &serde_json::Value) {
        assert!(!self.authored_data_store.get(&data_address).is_some());
        assert!(!self.data_store.get(&data_address).is_some());
        self.data_store
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
    pub fn reply_fetch_data(&mut self, request: &FetchDhtData) -> NetResult<()> {
        let msg;
        {
            // Get data from local datastores
            let mut maybe_data = self.authored_data_store.get(&request.data_address);
            if maybe_data.is_none() {
                maybe_data = self.data_store.get(&request.data_address);
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
                    let msg_data = HandleDhtResultData {
                        request_id: request.request_id.clone(),
                        requester_agent_id: request.requester_agent_id.clone(),
                        dna_address: request.dna_address.clone(),
                        provider_agent_id: self.agent_id.clone(),
                        data_address: request.data_address.clone(),
                        data_content: data.clone(),
                    };
                    JsonProtocol::HandleFetchDhtDataResult(msg_data).into()
                }
            };
        }
        println!("({}) reply_fetch_data: {:?}", self.agent_id, msg);
        self.send(msg)
    }

    /// Send a reponse to a FetchDhtMetaData request
    pub fn reply_fetch_meta(&mut self, request: &FetchDhtMetaData) -> NetResult<()> {
        let msg;
        {
            // Get meta from local datastores
            let meta_pair = &(request.data_address.clone(), request.attribute.clone());
            let mut maybe_data = self.authored_meta_store.get(meta_pair);
            if maybe_data.is_none() {
                maybe_data = self.meta_store.get(meta_pair);
            }
            // if meta not found send empty content (will make the aggregation easier)
            let data = match maybe_data.clone() {
                Some(data) => data.clone(),
                None => json!(""),
            };
            msg = HandleDhtMetaResultData {
                request_id: request.request_id.clone(),
                requester_agent_id: request.requester_agent_id.clone(),
                dna_address: request.dna_address.clone(),
                provider_agent_id: self.agent_id.clone(),
                data_address: request.data_address.clone(),
                attribute: request.attribute.clone(),
                content: data.clone(),
            };
        }
        println!("({}) reply_fetch_meta: {:?}", self.agent_id, msg);
        self.send(JsonProtocol::HandleFetchDhtMetaResult(msg).into())
    }

    // -- LISTS -- //

    pub fn reply_get_publish_data_list(&mut self, request: &GetListData) -> NetResult<()> {
        let data_address_list = self
            .authored_data_store
            .iter()
            .map(|(k, _)| k.clone())
            .collect();
        let msg = HandleListResultData {
            data_address_list,
            request_id: request.request_id.clone(),
            dna_address: request.dna_address.clone(),
        };
        self.send(JsonProtocol::HandleGetPublishingDataListResult(msg).into())
    }

    pub fn reply_get_publish_meta_list(&mut self, request: &GetListData) -> NetResult<()> {
        let meta_list = self
            .authored_meta_store
            .iter()
            .map(|(k, _)| k.clone())
            .collect();
        let msg = HandleMetaListResultData {
            meta_list,
            request_id: request.request_id.clone(),
            dna_address: request.dna_address.clone(),
        };
        self.send(JsonProtocol::HandleGetPublishingMetaListResult(msg).into())
    }

    pub fn reply_get_holding_data_list(&mut self, request: &GetListData) -> NetResult<()> {
        let data_address_list = self.data_store.iter().map(|(k, _)| k.clone()).collect();
        let msg = HandleListResultData {
            data_address_list,
            request_id: request.request_id.clone(),
            dna_address: request.dna_address.clone(),
        };
        self.send(JsonProtocol::HandleGetHoldingDataListResult(msg).into())
    }

    pub fn reply_get_holding_meta_list(&mut self, request: &GetListData) -> NetResult<()> {
        let meta_list = self.meta_store.iter().map(|(k, _)| k.clone()).collect();
        let msg = HandleMetaListResultData {
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
                println!("<<< P2pNode({}) handler: {:?}", agent_id_arg, r);
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
            data_store: HashMap::new(),
            meta_store: HashMap::new(),
            authored_data_store: HashMap::new(),
            authored_meta_store: HashMap::new(),
        }
    }

    // Constructor for an in-memory P2P Network
    #[cfg_attr(tarpaulin, skip)]
    pub fn new_with_unique_memory_network(agent_id: String) -> Self {
        let config = P2pConfig::new_with_unique_memory_backend();
        return P2pNode::new_with_config(agent_id, &config, None);
    }

    // Constructor for an IPC node that uses an existing n3h process and a temp folder
    #[cfg_attr(tarpaulin, skip)]
    pub fn new_with_uri_ipc_network(agent_id: String, ipc_binding: &str) -> Self {
        let p2p_config = P2pConfig::default_ipc_uri(Some(ipc_binding));
        return P2pNode::new_with_config(agent_id, &p2p_config, None);
    }

    // Constructor for an IPC node that spawns and uses a n3h process and a temp folder
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

    // See if there is a message to receive
    #[cfg_attr(tarpaulin, skip)]
    pub fn try_recv(&mut self) -> NetResult<JsonProtocol> {
        let data = self.receiver.try_recv()?;
        // Print non-ping messages
        match data {
            Protocol::NamedBinary(_) => println!("<< P2pNode({}) recv: {:?}", self.agent_id, data),
            Protocol::Json(_) => println!("<< P2pNode({}) recv: {:?}", self.agent_id, data),
            _ => (),
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

            if let Ok(p2p_msg) = self.try_recv() {
                println!(
                    "P2pNode({})::listen() - received: {:?}",
                    self.agent_id, p2p_msg
                );
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

    //    // look for a msg in log or wait for it
    //    #[cfg_attr(tarpaulin, skip)]
    //    pub fn has_or_wait(
    //        &mut self,
    //        ith: usize,
    //        predicate: Box<dyn Fn(&JsonProtocol) -> bool>,
    //        timeout_ms: usize,
    //    ) -> JsonProtocol {
    //        let maybe_msg = self.find_recv_msg(ith, predicate.clone());
    //        if maybe_msg.is_some() {
    //            return maybe_msg.unwrap();
    //        }
    //        self.wait_with_timeout(predicate, timeout_ms)
    //    }

    /// Wait for receiving a message corresponding to predicate until timeout is reached
    pub fn wait_with_timeout(
        &mut self,
        predicate: Box<dyn Fn(&JsonProtocol) -> bool>,
        timeout_ms: usize,
    ) -> JsonProtocol {
        let mut time_ms: usize = 0;
        loop {
            let mut did_something = false;

            if let Ok(p2p_msg) = self.try_recv() {
                println!(
                    "P2pNode({})::wait() - received: {:?}",
                    self.agent_id, p2p_msg
                );
                did_something = true;
                if predicate(&p2p_msg) {
                    println!("\t P2pNode({})::wait() - match", self.agent_id);
                    return p2p_msg;
                } else {
                    println!("\t P2pNode({})::wait() - NO match", self.agent_id);
                }
            }

            if !did_something {
                std::thread::sleep(std::time::Duration::from_millis(10));
                time_ms += 10;
                if time_ms > timeout_ms {
                    panic!("P2pNode({})::wait() has TIMEOUT", self.agent_id);
                }
            }
        }
    }

    /// Wait for receiving a message corresponding to predicate
    /// hard coded timeout
    #[cfg_attr(tarpaulin, skip)]
    pub fn wait(&mut self, predicate: Box<dyn Fn(&JsonProtocol) -> bool>) -> JsonProtocol {
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

            JsonProtocol::FetchDhtData(_) => {
                panic!("Core should not receive FetchDhtData message");
            }
            JsonProtocol::FetchDhtDataResult(_) => {
                // n/a
            }
            JsonProtocol::HandleFetchDhtData(_msg) => {
                // n/a
            }
            JsonProtocol::HandleFetchDhtDataResult(_msg) => {
                // n/a
            }

            JsonProtocol::PublishDhtData(_msg) => {
                panic!("Core should not receive PublishDhtData message");
            }
            JsonProtocol::HandleStoreDhtData(msg) => {
                // Store data in local datastore
                self.data_store.insert(msg.data_address, msg.data_content);
            }
            JsonProtocol::HandleDropDhtData(msg) => {
                // Remove data in local datastore
                self.data_store.remove(&msg.data_address);
            }

            JsonProtocol::FetchDhtMeta(_msg) => {
                panic!("Core should not receive FetchDhtMeta message");
            }
            JsonProtocol::FetchDhtMetaResult(_msg) => {
                // n/a
            }
            JsonProtocol::HandleFetchDhtMeta(_msg) => {
                // n/a
            }
            JsonProtocol::HandleFetchDhtMetaResult(_msg) => {
                // n/a
            }

            JsonProtocol::PublishDhtMeta(_msg) => {
                panic!("Core should not receive PublishDhtMeta message");
            }
            JsonProtocol::HandleStoreDhtMeta(msg) => {
                // Store data in local datastore
                self.meta_store
                    .insert((msg.data_address, msg.attribute), msg.content);
            }
            JsonProtocol::HandleDropMetaData(msg) => {
                // Remove data in local datastore
                self.meta_store.remove(&(msg.data_address, msg.attribute));
            }

            // -- Publish & Hold data -- //
            JsonProtocol::HandleGetPublishingDataList(_msg) => {
                // n/a
            }
            JsonProtocol::HandleGetPublishingDataListResult(_) => {
                panic!("Core should not receive HandleGetPublishingDataListResult message");
            }
            JsonProtocol::HandleGetHoldingDataList(_msg) => {
                // n/a
            }
            // Our request for the hold_list has returned
            JsonProtocol::HandleGetHoldingDataListResult(_) => {
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
        println!(">> P2pNode({}) send: {:?}", self.agent_id, data);
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
