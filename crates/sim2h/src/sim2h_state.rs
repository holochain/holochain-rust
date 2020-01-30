use crate::*;
use lib3h::rrdht_util::*;

pub(crate) struct Sim2hState {
    pub(crate) crypto: Box<dyn CryptoSystem>,
    pub(crate) connection_states: HashMap<Lib3hUri, ConnectionStateItem>,
    pub(crate) spaces: HashMap<SpaceHash, Space>,
    pub(crate) metric_gen: MetricsTimerGenerator,
    pub(crate) connection_mgr: ConnectionMgrHandle,
}

pub(crate) type ConnectionStateItem = (String, ConnectionState);

impl Sim2hState {
    // find out if an agent is in a space or not and return its URI
    pub(crate) fn lookup_joined(
        &self,
        space_address: &SpaceHash,
        agent_id: &AgentId,
    ) -> Option<Lib3hUri> {
        let _m = self.metric_gen.timer("sim2h-state-lookup_joined");
        self.spaces.get(space_address)?.agent_id_to_uri(agent_id)
    }

    // removes an agent from a space
    pub(crate) fn leave(&mut self, uri: &Lib3hUri, data: &SpaceData) -> Sim2hResult<()> {
        let _m = self.metric_gen.timer("sim2h-state-leave");
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
        let _m = self.metric_gen.timer("sim2h-state-get_or_create_space");
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
        space
    }

    /// removes a uri from connection and from spaces
    pub(crate) fn disconnect(&mut self, uri: &Lib3hUri) {
        let _m = self.metric_gen.timer("sim2h-state-disconnect");
        trace!("disconnect entered");

        self.connection_mgr.disconnect(uri.clone());

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
            "Space {} now knows about {} entries:\n",
            &space_address,
            space.all_aspects().len()
        );
        trace!(
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
        let wire_message = WireMessage::Lib3hToClient(
            ht::top_follower("request_gossiping_list")
                .wrap(Lib3hToClient::HandleGetGossipingEntryList(GetListData {
                    request_id: "".into(),
                    space_address,
                    provider_agent_id: provider_agent_id.clone(),
                }))
                .into(),
        );
        self.send(provider_agent_id, uri, &wire_message);
    }

    #[autotrace]
    pub(crate) fn request_authoring_list(
        &mut self,
        uri: Lib3hUri,
        space_address: SpaceHash,
        provider_agent_id: AgentId,
    ) {
        let span = ht::top_follower("inner");
        let msg = Lib3hToClient::HandleGetAuthoringEntryList(GetListData {
            request_id: "".into(),
            space_address,
            provider_agent_id: provider_agent_id.clone(),
        });
        let wire_message = WireMessage::Lib3hToClient(span.wrap(msg).into());
        self.send(provider_agent_id, uri, &wire_message);
    }

    #[autotrace]
    pub(crate) fn send(&self, agent: AgentId, uri: Lib3hUri, msg: &WireMessage) -> Vec<Lib3hUri> {
        let _m = self.metric_gen.timer("sim2h-state-send");

        match msg {
            _ => {
                debug!(">>OUT>> {} to {}", msg.message_type(), uri);
                MESSAGE_LOGGER
                    .lock()
                    .log_out(agent, uri.clone(), msg.clone());
            }
        }

        let payload: Opaque = msg.clone().into();
        self.connection_mgr
            .send_data(uri, payload.as_bytes().into());

        match msg {
            WireMessage::Ping | WireMessage::Pong => {}
            _ => debug!("sent."),
        }

        vec![]
    }

    pub(crate) fn retry_sync_missing_aspects(&mut self) {
        let _m = self
            .metric_gen
            .timer("sim2h-state-retry_sync_missing_aspects");
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
        let _m = self
            .metric_gen
            .timer("sim2h-state-get_agent_not_missing_aspects");
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

    #[autotrace]
    pub(crate) fn build_query(
        &self,
        space_address: SpaceHash,
        query_data: QueryEntryData,
        redundant_count: u64,
    ) -> Vec<Lib3hUri> {
        let _m = self.metric_gen.timer("sim2h-state-build_query");

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
        let span = ht::top_follower("inner");
        let query_message = WireMessage::Lib3hToClient(
            span.wrap(Lib3hToClient::HandleQueryEntry(query_data))
                .into(),
        );
        self.send(query_target, url, &query_message)
    }

    #[autotrace]
    pub(crate) fn build_aspects_from_arbitrary_agent(
        &self,
        aspects_to_fetch: AspectList,
        for_agent_id: AgentId,
        mut agent_pool: Vec<AgentId>,
        space_address: SpaceHash,
    ) -> Vec<Lib3hUri> {
        let _m = self
            .metric_gen
            .timer("sim2h-state-build_aspects_from_arbitrary_agent");
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

                    let msg = Lib3hToClient::HandleFetchEntry(FetchEntryData {
                        request_id: for_agent_id.clone().into(),
                        space_address: space_address.clone(),
                        provider_agent_id: arbitrary_agent.clone(),
                        entry_address: entry_address.clone(),
                        aspect_address_list: Some(aspect_address_list.clone()),
                    });
                    let wire_message =
                        WireMessage::Lib3hToClient(ht::top_follower("inner").wrap(msg).into());
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

    // get the connection status of an agent
    pub(crate) fn get_connection(&self, uri: &Lib3hUri) -> Option<ConnectionStateItem> {
        self.connection_states.get(uri).map(|ca| (*ca).clone())
    }

    #[autotrace]
    pub(crate) fn build_handle_unseen_aspects(
        &self,
        uri: Lib3hUri,
        space_address: SpaceHash,
        agent_id: AgentId,
        list_data: EntryListData,
    ) -> Vec<Lib3hUri> {
        let _m = self
            .metric_gen
            .timer("sim2h-state-build_handle_unseen_aspects");
        let unseen_aspects = AspectList::from(list_data.address_map)
            .diff(self.get_space(&space_address).all_aspects());
        let mut disconnects = Vec::new();
        if unseen_aspects.len() > 0 {
            debug!("UNSEEN ASPECTS:\n{}", unseen_aspects.pretty_string());
            let mut multi_messages = Vec::new();
            for entry_address in unseen_aspects.entry_addresses() {
                if let Some(aspect_address_list) = unseen_aspects.per_entry(entry_address) {
                    let msg = Lib3hToClient::HandleFetchEntry(FetchEntryData {
                        request_id: "".into(),
                        space_address: space_address.clone(),
                        provider_agent_id: agent_id.clone(),
                        entry_address: entry_address.clone(),
                        aspect_address_list: Some(aspect_address_list.clone()),
                    });
                    multi_messages.push(ht::top_follower("inner").wrap(msg).into());
                }
            }
            let multi_message = WireMessage::MultiSend(multi_messages);
            disconnects.append(&mut self.send(agent_id, uri, &multi_message));
        } else {
            debug!("NO UNSEEN ASPECTS")
        }
        disconnects
    }

    #[autotrace]
    pub(crate) async fn handle_new_entry_data(
        sim2h_handle: Sim2hHandle,
        entry_data: EntryData,
        space_address: SpaceHash,
        provider: AgentPubKey,
    ) {
        let _m = sim2h_handle.metric_timer("sim2h-state-build_handle_new_entry_data");
        let aspect_addresses = entry_data
            .aspect_list
            .iter()
            .cloned()
            .map(|aspect_data| aspect_data.aspect_address)
            .collect::<Vec<_>>();
        let mut map = HashMap::new();
        map.insert(entry_data.entry_address.clone(), aspect_addresses);
        let aspect_list = AspectList::from(map);
        debug!("GOT NEW ASPECTS:\n{}", aspect_list.pretty_string());

        let mut to_add = Vec::new();
        let mut multi_messages = Vec::new();
        for aspect in entry_data.aspect_list {
            // 1. Add hashes to our global list of all aspects in this space:
            to_add.push((
                entry_data.entry_address.clone(),
                aspect.aspect_address.clone(),
            ));

            // 2. Create store message
            let msg = Lib3hToClient::HandleStoreEntryAspect(StoreEntryAspectData {
                request_id: "".into(),
                space_address: space_address.clone(),
                provider_agent_id: provider.clone(),
                entry_address: entry_data.entry_address.clone(),
                entry_aspect: aspect,
            });
            multi_messages.push(ht::top_follower("inner").wrap(msg).into());
        }
        let multi_message = WireMessage::MultiSend(multi_messages);

        // 3. Send store message to selected nodes
        let mut state = sim2h_handle.lock_state().await;

        // Calculate list of agents that should store new data:
        let dht_agents = match sim2h_handle.dht_algorithm() {
            DhtAlgorithm::FullSync => {
                state.all_agents_except_one(space_address.clone(), Some(&provider))
            }
            DhtAlgorithm::NaiveSharding { redundant_count } => {
                let entry_loc = entry_location(&state.crypto, &entry_data.entry_address);
                state.agents_in_neighbourhood(space_address.clone(), entry_loc, *redundant_count)
            }
        };

        for (entry_address, aspect_address) in to_add.drain(..) {
            state.add_aspect(&space_address, entry_address, aspect_address);
        }

        state.broadcast(&multi_message, dht_agents);
    }

    pub(crate) fn broadcast(&mut self, msg: &WireMessage, agents: Vec<(AgentId, AgentInfo)>) {
        let _m = self.metric_gen.timer("sim2h-state-broadcast");
        for (agent, info) in agents {
            debug!("Broadcast: Sending to {:?}", info.uri);
            self.send(agent, info.uri, msg);
        }
    }

    pub(crate) fn all_agents_except_one(
        &self,
        space: SpaceHash,
        except: Option<&AgentId>,
    ) -> Vec<(AgentId, AgentInfo)> {
        let _m = self.metric_gen.timer("sim2h-state-all_agents_except_one");
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
        let _m = self.metric_gen.timer("sim2h-state-agents_in_neighbourhood");
        self.get_space(&space)
            .agents_supposed_to_hold_entry(entry_loc, redundant_count)
            .into_iter()
            .collect::<Vec<(AgentId, AgentInfo)>>()
    }
}
