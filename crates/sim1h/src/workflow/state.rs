use crate::dht::bbdht::dynamodb::{
    api::{
        agent::inbox::check_inbox,
        aspect::read::{get_aspect, scan_aspects},
        item::Item,
    },
    client::Client,
};
use lib3h_protocol::{
    data_types::{GetListData, StoreEntryAspectData},
    protocol::{ClientToLib3hResponse, Lib3hToClient},
    types::{AgentPubKey, AspectHash, EntryHash, SpaceHash},
};
use std::{
    collections::{
        hash_map::Entry::{Occupied, Vacant},
        HashMap, HashSet,
    },
    time::Instant,
};
use uuid::Uuid;

pub type AspectAddressMap = HashMap<EntryHash, HashSet<AspectHash>>;
type Sim1hResult<T> = Result<T, String>;

const MIN_TOLERABLE_TICK_INTERVAL_MS: u128 = 80;

#[derive(Default)]
pub struct Sim1hState {
    pub initialized: bool,
    pub space_hash: SpaceHash,
    pub agent_id: AgentPubKey,
    pub client_request_outbox: Vec<Lib3hToClient>,
    pub client_response_outbox: Vec<ClientToLib3hResponse>,
    pub held_aspects: AspectAddressMap,
    num_ticks: u32,
    last_tick_instant: Option<Instant>,
    last_evaluated_scan_key: Option<Item>,
}

impl Sim1hState {
    pub fn new(space_hash: SpaceHash, agent_id: AgentPubKey) -> Self {
        Self {
            space_hash,
            agent_id,
            ..Self::default()
        }
    }

    fn should_get_authoring_list(&mut self) -> bool {
        !self.initialized
    }

    fn create_authoring_gossip_list_requests(&self) -> Vec<Lib3hToClient> {
        let mut requests = Vec::new();
        requests.push(Lib3hToClient::HandleGetAuthoringEntryList(GetListData {
            space_address: self.space_hash.clone(),
            provider_agent_id: self.agent_id.clone(),
            request_id: "".into(),
        }));
        requests.push(Lib3hToClient::HandleGetGossipingEntryList(GetListData {
            space_address: self.space_hash.clone(),
            provider_agent_id: self.agent_id.clone(),
            request_id: "".into(),
        }));

        requests
    }

    fn create_direct_message_requests(&self, client: &Client) -> Vec<Lib3hToClient> {
        if !self.initialized {
            return Vec::new();
        }
        let log_context = "Sim1hState::create_direct_message_requests";
        match check_inbox(
            &log_context,
            client,
            &self.space_hash.clone().into(),
            &self.agent_id,
        ) {
            Ok(direct_messages) => direct_messages
                .into_iter()
                .map(|(message, is_response)| {
                    if is_response {
                        Lib3hToClient::SendDirectMessageResult(message)
                    } else {
                        Lib3hToClient::HandleSendDirectMessage(message)
                    }
                })
                .collect(),
            Err(error) => {
                error!("Error checking inbox: {:?}", error);
                Vec::new()
            }
        }
    }

    fn create_store_requests(&mut self, client: &Client) -> Sim1hResult<Vec<Lib3hToClient>> {
        if !self.initialized {
            return Ok(Vec::new());
        }

        let log_context = "create_store_requests";
        let agent_id = self.agent_id.to_owned();
        let space_hash = self.space_hash.to_owned();
        let table_name = space_hash.clone().into();
        let (incoming, last_evaluated_key) = scan_aspects(
            log_context,
            client,
            &table_name,
            self.last_evaluated_scan_key.clone(),
        )
        .map_err(|err| err.to_string())?;
        self.last_evaluated_scan_key = last_evaluated_key;
        let mut messages = Vec::new();

        for entry_hash in incoming.keys() {
            let aspects = incoming[entry_hash].clone();
            let diff = match self.held_aspects.entry(entry_hash.clone()) {
                Vacant(e) => {
                    e.insert(aspects.clone());
                    aspects.into_iter().collect()
                }
                Occupied(mut e) => {
                    let old = e.insert(aspects.clone());
                    aspects.difference(&old).cloned().collect::<Vec<_>>()
                }
            };
            messages.append(
                &mut diff
                    .into_iter()
                    .filter_map(|aspect_address| {
                        get_aspect(&log_context, client, &table_name, &aspect_address)
                            .expect("Cannot get aspect")
                    })
                    .map(|entry_aspect| {
                        Lib3hToClient::HandleStoreEntryAspect(StoreEntryAspectData {
                            request_id: Uuid::new_v4().to_string(), // XXX: well, is this so bad?
                            space_address: space_hash.clone(),
                            provider_agent_id: agent_id.clone(), // TODO: is this OK?
                            entry_address: entry_hash.clone(),
                            entry_aspect,
                        })
                    })
                    .collect(),
            );
        }

        Ok(messages)
    }

    pub fn process_pending_requests_to_client(
        &mut self,
        client: &Client,
    ) -> Sim1hResult<Vec<Lib3hToClient>> {
        let requests = if self.should_get_authoring_list() {
            self.initialized = true;
            self.create_authoring_gossip_list_requests()
        } else {
            Vec::new()
        };

        let now = Instant::now();
        if let Some(then) = self.last_tick_instant {
            let millis = now.duration_since(then).as_millis();
            if millis < MIN_TOLERABLE_TICK_INTERVAL_MS {
                return Ok(Vec::new());
            }
        }
        self.last_tick_instant = Some(now);
        self.num_ticks += 1;

        let messages = requests
            .into_iter()
            .chain(self.create_store_requests(client)?.into_iter())
            .chain(self.create_direct_message_requests(client).into_iter())
            .chain(self.client_request_outbox.drain(..))
            .collect();

        Ok(messages)
    }

    pub fn process_pending_responses_to_client(&mut self) -> Vec<ClientToLib3hResponse> {
        self.client_response_outbox.drain(..).collect()
    }
}
