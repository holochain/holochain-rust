use crate::*;
use futures::{
    lock::Mutex,
};

pub struct Sim2hStateMutex {
    #[allow(dead_code)]
    ctx: Sim2hContextInner,
    #[allow(dead_code)]
    inner: Arc<Mutex<Sim2hState>>,
}

pub type Sim2hStateRef = Arc<Sim2hStateMutex>;

impl Sim2hStateMutex {
    pub fn new(ctx: Sim2hContextInner, state: Sim2hState) -> Sim2hStateRef {
        let inner = Arc::new(Mutex::new(state));
        let out = Arc::new(Sim2hStateMutex {
            ctx,
            inner: inner.clone(),
        });
        out
    }
}

pub struct Sim2hState {
    pub(crate) crypto: Box<dyn CryptoSystem>,
    pub(crate) connection_states: std::collections::HashMap<Lib3hUri, ConnectionStateItem>,
    pub(crate) open_connections: std::collections::HashMap<Lib3hUri, OpenConnectionItem>,
    pub(crate) spaces: HashMap<SpaceHash, Space>,
    pub(crate) metric_publisher: Arc<holochain_locksmith::RwLock<dyn MetricPublisher>>,
}

pub type ConnectionStateItem = (String, ConnectionState);

pub struct OpenConnectionItem {
    pub(crate) version: WireMessageVersion,
    pub(crate) uuid: String,
    // TODO dangerous mixing futures && classic mutexes - fix this
    pub(crate) job: Arc<holochain_locksmith::Mutex<ConnectionJob>>,
    pub(crate) sender: crossbeam_channel::Sender<WsFrame>,
}

impl Sim2hState {
    // find out if an agent is in a space or not and return its URI
    pub(crate) fn lookup_joined(&self, space_address: &SpaceHash, agent_id: &AgentId) -> Option<Lib3hUri> {
        with_latency_publishing!("sim2h-state-lookup_joined", self.metric_publisher, || {
            self.spaces.get(space_address)?.agent_id_to_uri(agent_id)
        })
    }

    // removes an agent from a space
    pub(crate) fn leave(&mut self, uri: &Lib3hUri, data: &SpaceData) -> Sim2hResult<()> {
        with_latency_publishing!("sim2h-disconnnect", self.metric_publisher, || {
            if let Some((uuid, state)) = self.get_connection(uri) {
                conn_lifecycle("leave -> disconnect", &uuid, &state, uri);
                if let ConnectionState::Joined(space_address, agent_id) = state {
                    if (data.agent_id != agent_id) || (data.space_address != space_address) {
                        Err(SPACE_MISMATCH_ERR_STR.into())
                    } else {
                        self.disconnect(uri);
                        Ok(())
                    }
                } else {
                    Err(format!("no joined agent found at {} ", &uri).into())
                }
            } else {
                Err(format!("no agent found at {} ", &uri).into())
            }
        })
    }

    pub(crate) fn get_space(&self, space_address: &SpaceHash) -> &Space {
        self.spaces
            .get(space_address)
            .expect("we should be able to get the space")
    }

    pub(crate) fn get_space_mut(&mut self, space_address: &SpaceHash) -> &mut Space {
        self.spaces
            .get_mut(space_address)
            .expect("we should be able to get the space")
    }

    pub(crate) fn get_or_create_space(&mut self, space_address: &SpaceHash) -> &mut Space {
        let clock = std::time::SystemTime::now();
        let contains_space = self.spaces.contains_key(space_address);
        if !contains_space {
            self.spaces
                .insert(space_address.clone(), Space::new(self.crypto.box_clone()));
            info!(
                "\n\n+++++++++++++++\nNew Space: {}\n+++++++++++++++\n",
                space_address
            );
        }
        let space = self.spaces.get_mut(space_address).unwrap();
        self.metric_publisher
            .write()
            .unwrap()
            .publish(&Metric::new_timestamped_now(
                "sim2h-get_or_create_space.latency",
                None,
                clock.elapsed().unwrap().as_millis() as f64,
            ));
        space
    }
    // removes a uri from connection and from spaces
    pub(crate) fn disconnect(&mut self, uri: &Lib3hUri) {
        with_latency_publishing!("sim2h-disconnnect", self.metric_publisher, || {
            trace!("disconnect entered");

            if let Some(OpenConnectionItem {
                version: _,
                uuid,
                job: con,
                sender: _,
            }) = self.open_connections.remove(uri)
            {
                open_lifecycle("disconnect", &uuid, uri);
                con.f_lock().stop();
            }

            if let Some((uuid, conn)) = self.connection_states.remove(uri) {
                conn_lifecycle("disconnect", &uuid, &conn, uri);
                if let ConnectionState::Joined(space_address, agent_id) = conn {
                    if let Some(space) = self.spaces.get_mut(&space_address) {
                        if space.remove_agent(&agent_id) == 0 {
                            self.spaces.remove(&space_address);
                        }
                    }
                }
            }
            trace!("disconnect done");
        })
    }

    pub(crate) fn join_agent(
        &mut self,
        space_address: &SpaceHash,
        agent_id: AgentId,
        uri: Lib3hUri,
    ) -> Sim2hResult<()> {
        let space = self.get_or_create_space(space_address);
        space.join_agent(agent_id, uri)
    }

    pub(crate) fn add_missing_aspects(
        &mut self,
        space_address: &SpaceHash,
        agent_id: &AgentId,
        missing_hashes: HashSet<(EntryHash, AspectHash)>,
    ) {
        let space = self.get_or_create_space(space_address);
        for (entry_hash, aspect_hash) in missing_hashes {
            space.add_missing_aspect(agent_id.clone(), entry_hash, aspect_hash);
        }
    }

    pub(crate) fn add_aspect(
        &mut self,
        space_address: &SpaceHash,
        entry_hash: EntryHash,
        aspect_hash: AspectHash,
    ) {
        let space = self.get_or_create_space(space_address);
        space.add_aspect(entry_hash, aspect_hash);
        debug!(
            "Space {} now knows about these aspects:\n{}",
            &space_address,
            space.all_aspects().pretty_string()
        );
    }

    pub(crate) fn remove_missing_aspect(
        &mut self,
        space_address: &SpaceHash,
        agent_id: &AgentId,
        entry_hash: &EntryHash,
        aspect_hash: &AspectHash,
    ) {
        let space = self.get_space_mut(space_address);
        space.remove_missing_aspect(agent_id, entry_hash, aspect_hash);
    }

    pub(crate) fn request_gossiping_list(
        &mut self,
        uri: Lib3hUri,
        space_address: SpaceHash,
        provider_agent_id: AgentId,
    ) {
        with_latency_publishing!(
            "sim2h-request_gossiping_list",
            self.metric_publisher,
            || {
                let wire_message = WireMessage::Lib3hToClient(
                    Lib3hToClient::HandleGetGossipingEntryList(GetListData {
                        request_id: "".into(),
                        space_address,
                        provider_agent_id: provider_agent_id.clone(),
                    }),
                );
                self.send(provider_agent_id, uri, &wire_message);
            }
        )
    }

    pub(crate) fn send(&self, agent: AgentId, uri: Lib3hUri, msg: &WireMessage) -> Vec<Lib3hUri> {
        with_latency_publishing!("sim2h-send", self.metric_publisher, || {
            match msg {
                _ => {
                    debug!(">>OUT>> {} to {}", msg.message_type(), uri);
                    MESSAGE_LOGGER
                        .lock()
                        .log_out(agent, uri.clone(), msg.clone());
                }
            }

            let mut to_disconnect = Vec::new();

            match self.open_connections.get(&uri) {
                None => {
                    error!("FAILED TO SEND, NO ROUTE: {}", uri);
                    return to_disconnect;
                }
                Some(OpenConnectionItem {
                    version,
                    uuid,
                    job: _,
                    sender: outgoing_send,
                }) => {
                    open_lifecycle("send", uuid, &uri);

                    if (version > &mut 1)
                        || match msg {
                            WireMessage::MultiSend(_) => false,
                            _ => true,
                        }
                    {
                        let payload: Opaque = msg.clone().into();

                        if let Err(_) = outgoing_send.send(payload.as_bytes().into()) {
                            // pass the back out to be disconnected
                            to_disconnect.push(uri.clone());
                        }
                    } else {
                        // version 1 can't handle multi send so send them all individually
                        if let WireMessage::MultiSend(messages) = msg {
                            for msg in messages {
                                let payload: Opaque =
                                    WireMessage::Lib3hToClient(msg.clone()).into();
                                if let Err(_) = outgoing_send.send(payload.as_bytes().into()) {
                                    to_disconnect.push(uri.clone());
                                }
                            }
                        }
                    }
                }
            }

            match msg {
                WireMessage::Ping | WireMessage::Pong => {}
                _ => debug!("sent."),
            }

            return to_disconnect;
        })
    }

    pub(crate) fn retry_sync_missing_aspects(&mut self) {
        with_latency_publishing!(
            "sim2h-retry_sync_missing_aspects",
            self.metric_publisher,
            || {
                debug!("Checking for nodes with missing aspects to retry sync...");
                // Extract all needed info for the call to self.request_gossiping_list() below
                // as copies so we don't have to keep a reference to self.
                let spaces_with_agents_and_uris = self
                    .spaces
                    .iter()
                    .filter_map(|(space_hash, space)| {
                        let agents = space.agents_with_missing_aspects();
                        // If this space doesn't have any agents with missing aspects,
                        // ignore it:
                        if agents.is_empty() {
                            None
                        } else {
                            // For spaces with agents with missing aspects,
                            // annotate all agent IDs with their corresponding URI:
                            let agent_ids_with_uris: Vec<(AgentId, Lib3hUri)> = agents
                                .iter()
                                .filter_map(|agent_id| {
                                    space
                                        .agent_id_to_uri(agent_id)
                                        .map(|uri| (agent_id.clone(), uri))
                                })
                                .collect();

                            Some((space_hash.clone(), agent_ids_with_uris))
                        }
                    })
                    .collect::<HashMap<SpaceHash, Vec<_>>>();

                for (space_hash, agents) in spaces_with_agents_and_uris {
                    for (agent_id, uri) in agents {
                        debug!("Re-requesting gossip list from {} at {}", agent_id, uri);
                        self.request_gossiping_list(uri, space_hash.clone(), agent_id);
                    }
                }
            }
        )
    }

    /// Get an agent who has at least one of the aspects specified, and who is not the same as for_agent_id.
    /// `agent_pool` is expected to be randomly shuffled, to ensure that no hotspots are created.
    pub(crate) fn get_agent_not_missing_aspects(
        &self,
        entry_hash: &EntryHash,
        aspects: &Vec<AspectHash>,
        for_agent_id: &AgentId,
        agent_pool: &[AgentId],
        space_address: &SpaceHash,
    ) -> Option<AgentId> {
        let space = self.spaces.get(space_address)?;
        agent_pool
            .into_iter()
            // We ignore all agents that are missing all of the same aspects as well since
            // they can't help us.
            .find(|a| {
                **a != *for_agent_id && !space.agent_is_missing_all_aspects(*a, entry_hash, aspects)
            })
            .cloned()
    }

    pub(crate) fn build_query(
        &self,
        space_address: SpaceHash,
        query_data: QueryEntryData,
        redundant_count: u64,
    ) -> Vec<Lib3hUri> {
        let entry_loc = entry_location(&self.crypto, &query_data.entry_address);
        let agent_pool = self
            .get_space(&space_address)
            .agents_supposed_to_hold_entry(entry_loc, redundant_count)
            .keys()
            .cloned()
            .collect::<Vec<_>>();

        let query_target = if agent_pool.is_empty() {
            // If there is nobody we could ask, just send the query back
            query_data.requester_agent_id.clone()
        } else {
            let agents_with_all_aspects_for_entry = agent_pool
                .iter()
                .filter(|agent| {
                    !self
                        .get_space(&space_address)
                        .agent_is_missing_some_aspect_for_entry(agent, &query_data.entry_address)
                })
                .cloned()
                .collect::<Vec<AgentId>>();

            let mut agents_to_sample_from = if agents_with_all_aspects_for_entry.is_empty() {
                // If there is nobody who as all aspects of an entry, just
                // ask somebody of that shard:
                agent_pool
            } else {
                agents_with_all_aspects_for_entry
            };

            let agent_slice = &mut agents_to_sample_from[..];
            agent_slice.shuffle(&mut thread_rng());
            agent_slice[0].clone()
        };

        let maybe_url = self.lookup_joined(&space_address, &query_target);
        if maybe_url.is_none() {
            error!("Got FetchEntryResult with request id that is not a known agent id. I guess we lost that agent before we could deliver missing aspects.");
            return vec![];
        }
        let url = maybe_url.unwrap();
        let query_message = WireMessage::Lib3hToClient(Lib3hToClient::HandleQueryEntry(query_data));
        self.send(query_target, url, &query_message)
    }

    pub(crate) fn build_aspects_from_arbitrary_agent(
        &self,
        aspects_to_fetch: AspectList,
        for_agent_id: AgentId,
        mut agent_pool: Vec<AgentId>,
        space_address: SpaceHash,
    ) -> Vec<Lib3hUri> {
        with_latency_publishing!(
            "sim2h-build_aspects_from_arbitrary_agent",
            self.metric_publisher,
            || {
                let agent_pool = &mut agent_pool[..];
                agent_pool.shuffle(&mut thread_rng());
                let mut disconnects = Vec::new();
                for entry_address in aspects_to_fetch.entry_addresses() {
                    if let Some(aspect_address_list) = aspects_to_fetch.per_entry(entry_address) {
                        if let Some(arbitrary_agent) = self.get_agent_not_missing_aspects(
                            entry_address,
                            aspect_address_list,
                            &for_agent_id,
                            agent_pool,
                            &space_address,
                        ) {
                            debug!(
                                "FETCHING missing contents from RANDOM AGENT: {}",
                                arbitrary_agent
                            );

                            let maybe_url = self.lookup_joined(&space_address, &arbitrary_agent);
                            if maybe_url.is_none() {
                                error!("Could not find URL for randomly selected agent. This should not happen!");
                                return Vec::new();
                            }
                            let random_url = maybe_url.unwrap();

                            let wire_message = WireMessage::Lib3hToClient(
                                Lib3hToClient::HandleFetchEntry(FetchEntryData {
                                    request_id: for_agent_id.clone().into(),
                                    space_address: space_address.clone(),
                                    provider_agent_id: arbitrary_agent.clone(),
                                    entry_address: entry_address.clone(),
                                    aspect_address_list: Some(aspect_address_list.clone()),
                                }),
                            );
                            debug!("SENDING fetch with request ID: {:?}", wire_message);
                            disconnects.append(&mut self.send(
                                arbitrary_agent.clone(),
                                random_url.clone(),
                                &wire_message,
                            ));
                        } else {
                            warn!("Could not find an agent that has any of the missing aspects. Trying again later...")
                        }
                    }
                }
                disconnects
            }
        )
    }

    // get the connection status of an agent
    pub(crate) fn get_connection(&self, uri: &Lib3hUri) -> Option<ConnectionStateItem> {
        with_latency_publishing!("sim2h-state-get_connection", self.metric_publisher, || {
            self.connection_states.get(uri).map(|ca| (*ca).clone())
        })
    }

    pub(crate) fn build_handle_unseen_aspects(
        &self,
        uri: Lib3hUri,
        space_address: SpaceHash,
        agent_id: AgentId,
        list_data: EntryListData,
    ) -> Vec<Lib3hUri> {
        with_latency_publishing!(
            "sim2h-build-handle-unseen_aspects",
            self.metric_publisher,
            || {
                let unseen_aspects = AspectList::from(HashMap::from(list_data.address_map))
                    .diff(self.get_space(&space_address).all_aspects());
                let mut disconnects = Vec::new();
                if unseen_aspects.len() > 0 {
                    debug!("UNSEEN ASPECTS:\n{}", unseen_aspects.pretty_string());
                    let mut multi_messages = Vec::new();
                    for entry_address in unseen_aspects.entry_addresses() {
                        if let Some(aspect_address_list) = unseen_aspects.per_entry(entry_address) {
                            multi_messages.push(Lib3hToClient::HandleFetchEntry(FetchEntryData {
                                request_id: "".into(),
                                space_address: space_address.clone(),
                                provider_agent_id: agent_id.clone(),
                                entry_address: entry_address.clone(),
                                aspect_address_list: Some(aspect_address_list.clone()),
                            }));
                        }
                    }
                    let multi_message = WireMessage::MultiSend(multi_messages);
                    disconnects.append(&mut self.send(
                        agent_id.clone(),
                        uri.clone(),
                        &multi_message,
                    ));
                } else {
                    debug!("NO UNSEEN ASPECTS")
                }
                disconnects
            }
        )
    }

    pub(crate) fn handle_new_entry_data(
        &mut self,
        entry_data: EntryData,
        space_address: SpaceHash,
        provider: AgentPubKey,
        dht_algorithm: DhtAlgorithm,
    ) {
        with_latency_publishing!("sim2h-handle_new_entry_data", self.metric_publisher, || {
            // Calculate list of agents that should store new data:
            let dht_agents = match dht_algorithm {
                DhtAlgorithm::FullSync => {
                    self.all_agents_except_one(space_address.clone(), Some(&provider))
                }
                DhtAlgorithm::NaiveSharding { redundant_count } => {
                    let entry_loc = entry_location(&self.crypto, &entry_data.entry_address);
                    self.agents_in_neighbourhood(space_address.clone(), entry_loc, redundant_count)
                }
            };

            let aspect_addresses = entry_data
                .aspect_list
                .iter()
                .cloned()
                .map(|aspect_data| aspect_data.aspect_address)
                .collect::<Vec<_>>();
            let mut map = HashMap::new();
            map.insert(entry_data.entry_address.clone(), aspect_addresses);
            let aspect_list = AspectList::from(HashMap::from(map));
            debug!("GOT NEW ASPECTS:\n{}", aspect_list.pretty_string());

            let mut multi_messages = Vec::new();
            for aspect in entry_data.aspect_list {
                // 1. Add hashes to our global list of all aspects in this space:
                self.add_aspect(
                    &space_address,
                    entry_data.entry_address.clone(),
                    aspect.aspect_address.clone(),
                );

                // 2. Create store message
                multi_messages.push(Lib3hToClient::HandleStoreEntryAspect(
                    StoreEntryAspectData {
                        request_id: "".into(),
                        space_address: space_address.clone(),
                        provider_agent_id: provider.clone(),
                        entry_address: entry_data.entry_address.clone(),
                        entry_aspect: aspect,
                    },
                ));
            }
            let multi_message = WireMessage::MultiSend(multi_messages);

            // 3. Send store message to selected nodes
            self.broadcast(&multi_message, dht_agents);
        })
    }

    pub(crate) fn broadcast(&mut self, msg: &WireMessage, agents: Vec<(AgentId, AgentInfo)>) {
        with_latency_publishing!("sim2h-broadcast", self.metric_publisher, || {
            for (agent, info) in agents {
                debug!("Broadcast: Sending to {:?}", info.uri);
                self.send(agent, info.uri, msg);
            }
        })
    }

    pub(crate) fn all_agents_except_one(
        &self,
        space: SpaceHash,
        except: Option<&AgentId>,
    ) -> Vec<(AgentId, AgentInfo)> {
        self.get_space(&space)
            .all_agents()
            .clone()
            .into_iter()
            .filter(|(a, _)| {
                if let Some(exception) = except {
                    *a != *exception
                } else {
                    true
                }
            })
            .collect::<Vec<(AgentId, AgentInfo)>>()
    }

    pub(crate) fn agents_in_neighbourhood(
        &self,
        space: SpaceHash,
        entry_loc: Location,
        redundant_count: u64,
    ) -> Vec<(AgentId, AgentInfo)> {
        self.get_space(&space)
            .agents_supposed_to_hold_entry(entry_loc, redundant_count)
            .into_iter()
            .collect::<Vec<(AgentId, AgentInfo)>>()
    }
}
