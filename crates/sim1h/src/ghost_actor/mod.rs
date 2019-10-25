use crate::dht::bbdht::dynamodb::client::{client, Client};
use crate::workflow::from_client::bootstrap::bootstrap;
use crate::workflow::from_client::fetch_entry::fetch_entry;
use crate::workflow::from_client::hold_entry::hold_entry;
use crate::workflow::from_client::join_space::join_space;
use crate::workflow::from_client::leave_space::leave_space;
use crate::workflow::from_client::publish_entry::publish_entry;
use crate::workflow::from_client::query_entry::query_entry;
use crate::workflow::from_client::send_direct_message::send_direct_message;
use crate::workflow::to_client::connected::connected;
use crate::workflow::to_client::disconnected::disconnected;
use crate::workflow::to_client::handle_drop_entry::handle_drop_entry;
use crate::workflow::to_client::handle_fetch_entry::handle_fetch_entry;
use crate::workflow::to_client::handle_get_authoring_entry_list::handle_get_authoring_entry_list;
use crate::workflow::to_client::handle_get_gossiping_entry_list::handle_get_gossiping_entry_list;
use crate::workflow::to_client::handle_query_entry::handle_query_entry;
use crate::workflow::to_client::handle_send_direct_message::handle_send_direct_message;
use crate::workflow::to_client::handle_store_entry_aspect::handle_store_entry_aspect;
use crate::workflow::to_client::send_direct_message_result::send_direct_message_result;
use crate::workflow::to_client_response::handle_drop_entry_result::handle_drop_entry_result;
use crate::workflow::to_client_response::handle_fetch_entry_result::handle_fetch_entry_result;
use crate::workflow::to_client_response::handle_get_authoring_entry_list_result::handle_get_authoring_entry_list_result;
use crate::workflow::to_client_response::handle_get_gossiping_entry_list_result::handle_get_gossiping_entry_list_result;
use crate::workflow::to_client_response::handle_query_entry_result::handle_query_entry_result;
use crate::workflow::to_client_response::handle_send_direct_message_result::handle_send_direct_message_result;
use crate::workflow::to_client_response::handle_store_entry_aspect_result::handle_store_entry_aspect_result;
use detach::Detach;
use lib3h::engine::engine_actor::ClientToLib3hMessage;
use lib3h::engine::CanAdvertise;
use lib3h::error::Lib3hError;
use lib3h::error::Lib3hResult;
use lib3h_protocol::protocol::ClientToLib3h;
use lib3h_protocol::protocol::ClientToLib3hResponse;
use lib3h_protocol::protocol::Lib3hToClient;
use lib3h_protocol::protocol::Lib3hToClientResponse;
use lib3h_zombie_actor::create_ghost_channel;
use lib3h_zombie_actor::GhostActor;
use lib3h_zombie_actor::GhostCanTrack;
use lib3h_zombie_actor::GhostContextEndpoint;
use lib3h_zombie_actor::GhostEndpoint;
use lib3h_zombie_actor::GhostResult;
use lib3h_zombie_actor::WorkWasDone;
use rusoto_core::Region;
use url::Url;

const REQUEST_ID_PREFIX: &str = "sim";

pub struct SimGhostActor {
    pub lib3h_endpoint: Detach<
        GhostContextEndpoint<
            SimGhostActor,
            Lib3hToClient,
            Lib3hToClientResponse,
            ClientToLib3h,
            ClientToLib3hResponse,
            Lib3hError,
        >,
    >,
    pub client_endpoint: Option<
        GhostEndpoint<
            ClientToLib3h,
            ClientToLib3hResponse,
            Lib3hToClient,
            Lib3hToClientResponse,
            Lib3hError,
        >,
    >,
    #[allow(dead_code)]
    dbclient: Client,
}

impl SimGhostActor {
    pub fn new(endpoint: &String) -> Self {
        let (endpoint_parent, endpoint_self) = create_ghost_channel();
        Self {
            client_endpoint: Some(endpoint_parent),
            lib3h_endpoint: Detach::new(
                endpoint_self
                    .as_context_endpoint_builder()
                    .request_id_prefix(REQUEST_ID_PREFIX)
                    .build(),
            ),
            dbclient: client(Region::Custom {
                name: "".to_string(),
                endpoint: endpoint.to_string(),
            }),
        }
    }

    pub fn be_cranked(&mut self, from_network: Lib3hToClient) -> Lib3hResult<()> {
        Ok(match from_network {
            Lib3hToClient::Connected(connected_data) => {
                let log_context = "Lib3hToClient::Connected";
                connected(&log_context, &self.dbclient, &connected_data);
            }
            Lib3hToClient::Disconnected(disconnected_data) => {
                let log_context = "Lib3hToClient::Disconnected";
                disconnected(&log_context, &self.dbclient, &disconnected_data);
            }
            Lib3hToClient::SendDirectMessageResult(direct_message_data) => {
                let log_context = "Lib3hToClient::SendDirectMessageResult";
                send_direct_message_result(&log_context, &self.dbclient, &direct_message_data)?;
            }
            // TODO
            Lib3hToClient::HandleSendDirectMessage(direct_message_data) => {
                let log_context = "Lib3hToClient::HandleSendDirectMessage";
                handle_send_direct_message(&log_context, &self.dbclient, &direct_message_data);
            }
            Lib3hToClient::HandleFetchEntry(fetch_entry_data) => {
                let log_context = "Lib3hToClient::HandleFetchEntry";
                handle_fetch_entry(&log_context, &self.dbclient, &fetch_entry_data);
            }
            Lib3hToClient::HandleStoreEntryAspect(store_entry_aspect_data) => {
                let log_context = "Lib3hToClient::HandleStoreEntryAspect";
                handle_store_entry_aspect(&log_context, &self.dbclient, &store_entry_aspect_data);
            }
            Lib3hToClient::HandleDropEntry(drop_entry_data) => {
                let log_context = "Lib3hToClient::HandleDropEntry";
                handle_drop_entry(&log_context, &self.dbclient, &drop_entry_data);
            }
            Lib3hToClient::HandleQueryEntry(query_entry_data) => {
                let log_context = "Lib3hToClient::HandleQueryEntry";
                handle_query_entry(&log_context, &self.dbclient, &query_entry_data);
            }
            Lib3hToClient::HandleGetAuthoringEntryList(get_list_data) => {
                let log_context = "Lib3hToClient::HandleGetAuthoringEntryList";
                handle_get_authoring_entry_list(&log_context, &self.dbclient, &get_list_data);
            }
            Lib3hToClient::HandleGetGossipingEntryList(get_list_data) => {
                let log_context = "Lib3hToClient::HandleGetGossipingEntryList";
                handle_get_gossiping_entry_list(&log_context, &self.dbclient, &get_list_data);
            }
        })
    }

    pub fn handle_client_response(from_client: Lib3hToClientResponse) {
        match from_client {
            // TOD
            Lib3hToClientResponse::HandleSendDirectMessageResult(direct_message_data) => {
                let log_context = "Lib3hToClientResponse::HandleSendDirectMessageResult";
                handle_send_direct_message_result(&log_context, &direct_message_data);
            }
            Lib3hToClientResponse::HandleFetchEntryResult(fetch_entry_result_data) => {
                let log_context = "Lib3hToClientResponse::HandleFetchEntryResult";
                handle_fetch_entry_result(&log_context, &fetch_entry_result_data);
            }
            Lib3hToClientResponse::HandleStoreEntryAspectResult => {
                let log_context = "Lib3hToClientResponse::HandleStoreEntryAspectResult";
                handle_store_entry_aspect_result(&log_context);
            }
            Lib3hToClientResponse::HandleDropEntryResult => {
                let log_context = "Lib3hToClientResponse::HandleDropEntryResult";
                handle_drop_entry_result(&log_context);
            }
            Lib3hToClientResponse::HandleQueryEntryResult(query_entry_result_data) => {
                let log_context = "Lib3hToClientResponse::HandleQueryEntryResult";
                handle_query_entry_result(&log_context, &query_entry_result_data);
            }
            Lib3hToClientResponse::HandleGetAuthoringEntryListResult(entry_list_data) => {
                let log_context = "Lib3hToClientResponse::HandleGetAuthoringEntryListResult";
                handle_get_authoring_entry_list_result(&log_context, &entry_list_data);
            }
            Lib3hToClientResponse::HandleGetGossipingEntryListResult(entry_list_data) => {
                let log_context = "Lib3hToClientResponse::HandleGetGossipingEntryListResult";
                handle_get_gossiping_entry_list_result(&log_context, &entry_list_data);
            }
        }
    }

    pub fn handle_msg_from_client(
        &mut self,
        mut msg: ClientToLib3hMessage,
    ) -> GhostResult<WorkWasDone> {
        match msg.take_message().expect("exists") {
            ClientToLib3h::Bootstrap(_) => {
                let log_context = "ClientToLib3h::Bootstrap";
                msg.respond(bootstrap(&log_context, &self.dbclient))?;
                Ok(true.into())
            }
            ClientToLib3h::JoinSpace(data) => {
                let log_context = "ClientToLib3h::JoinSpace";
                msg.respond(join_space(&log_context, &self.dbclient, &data))?;
                Ok(true.into())
            }
            ClientToLib3h::LeaveSpace(data) => {
                let log_context = "ClientToLib3h::LeaveSpace";
                msg.respond(leave_space(&log_context, &self.dbclient, &data))?;
                Ok(true.into())
            }
            // TODO - tests
            ClientToLib3h::SendDirectMessage(data) => {
                let log_context = "ClientToLib3h::SendDirectMessage";
                msg.respond(send_direct_message(&log_context, &self.dbclient, &data))?;
                Ok(true.into())
            }
            ClientToLib3h::PublishEntry(data) => {
                let log_context = "ClientToLib3h::PublishEntry";
                // no response message for publish entry
                publish_entry(&log_context, &self.dbclient, &data)?;
                Ok(true.into())
            }
            ClientToLib3h::HoldEntry(data) => {
                let log_context = "ClientToLib3h::HoldEntry";
                // no response message for hold entry
                hold_entry(&log_context, &self.dbclient, &data)?;
                Ok(true.into())
            }
            // TODO - query filtering needs work
            ClientToLib3h::QueryEntry(data) => {
                let log_context = "ClientToLib3h::QueryEntry";
                msg.respond(query_entry(&log_context, &self.dbclient, &data))?;
                Ok(true.into())
            }
            // TODO - tests & review assumption that this can just wrap QueryEntry
            ClientToLib3h::FetchEntry(data) => {
                let log_context = "ClientToLib3h::FetchEntry";
                msg.respond(fetch_entry(&log_context, &self.dbclient, &data))?;
                Ok(true.into())
            }
        }
    }
}

impl CanAdvertise for SimGhostActor {
    fn advertise(&self) -> Url {
        Url::parse("ws://example.com").unwrap()
    }
}

impl<'engine>
    GhostActor<
        Lib3hToClient,
        Lib3hToClientResponse,
        ClientToLib3h,
        ClientToLib3hResponse,
        Lib3hError,
    > for SimGhostActor
{
    /// our parent gets a reference to the parent side of our channel
    fn take_parent_endpoint(
        &mut self,
    ) -> Option<
        GhostEndpoint<
            ClientToLib3h,
            ClientToLib3hResponse,
            Lib3hToClient,
            Lib3hToClientResponse,
            Lib3hError,
        >,
    > {
        std::mem::replace(&mut self.client_endpoint, None)
    }

    /// we, as a ghost actor implement this, it will get called from
    /// process after the subconscious process items have run
    fn process_concrete(&mut self) -> GhostResult<WorkWasDone> {
        // always run the endpoint process loop
        detach_run!(&mut self.lib3h_endpoint, |cs| { cs.process(self) })?;

        let mut work_was_done = false;
        // process any messages from the client to us
        for msg in self.lib3h_endpoint.as_mut().drain_messages() {
            match self.handle_msg_from_client(msg) {
                Ok(msg_work_was_done) => work_was_done = work_was_done || msg_work_was_done.into(),
                Err(err) => return Err(err),
            }
        }

        // Done
        Ok(work_was_done.into())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::dht::bbdht::dynamodb::client::local::LOCAL_ENDPOINT;
    use holochain_tracing::test_span;
    use lib3h_protocol::{data_types::*, Address};
    use lib3h_zombie_actor::GhostCallbackData;

    fn get_response_to_request(
        mut engine: SimGhostActor,
        request: ClientToLib3h,
    ) -> GhostCallbackData<ClientToLib3hResponse, Lib3hError> {
        let mut parent_endpoint: GhostContextEndpoint<(), _, _, _, _, _> = engine
            .take_parent_endpoint()
            .expect("Could not get parent endpoint")
            .as_context_endpoint_builder()
            .request_id_prefix("parent")
            .build();

        let (s, r) = crossbeam_channel::unbounded();

        parent_endpoint
            .request(
                test_span(""),
                request,
                Box::new(move |_, callback_data| {
                    s.send(callback_data).unwrap();
                    Ok(())
                }),
            )
            .ok();

        for _ in 0..2 {
            // process a few times, once isn't enough..
            parent_endpoint.process(&mut ()).ok();
            engine.process().ok();
        }

        r.recv().expect("Could not retrieve result")
    }

    #[allow(dead_code)]
    fn get_response_to_request_threaded(
        request: ClientToLib3h,
    ) -> GhostCallbackData<ClientToLib3hResponse, Lib3hError> {
        let (s, r) = crossbeam_channel::unbounded();

        // TODO: maybe don't leave this thread running forever...
        std::thread::spawn(move || {
            let mut engine = SimGhostActor::new(&"invalid-endpoint".to_string());

            let mut parent_endpoint: GhostContextEndpoint<(), _, _, _, _, _> = engine
                .take_parent_endpoint()
                .expect("Could not get parent endpoint")
                .as_context_endpoint_builder()
                .request_id_prefix("parent")
                .build();

            parent_endpoint
                .request(
                    test_span(""),
                    request,
                    Box::new(move |_, callback_data| {
                        s.send(callback_data).unwrap();
                        Ok(())
                    }),
                )
                .ok();
            loop {
                parent_endpoint.process(&mut ()).ok();
                engine.process().ok();
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        });

        r.recv().expect("Could not retrieve result")
    }

    #[test]
    fn bootstrap_to_invalid_url_fails() {
        let engine = SimGhostActor::new(&"invalid-endpoint".to_string());

        let bootstrap_data = BootstrapData {
            space_address: Address::from(""),
            bootstrap_uri: Url::parse("http://fake_url").unwrap(),
        };

        match get_response_to_request(engine, ClientToLib3h::Bootstrap(bootstrap_data)) {
            GhostCallbackData::Response(Err(_)) => assert!(true),
            GhostCallbackData::Timeout => panic!("unexpected timeout"),
            r => panic!("unexpected response: {:?}", r),
        }
    }

    #[test]
    fn bootstrap_to_invalid_url_fails_threaded() {
        let bootstrap_data = BootstrapData {
            space_address: Address::from(""),
            bootstrap_uri: Url::parse("http://fake_url").unwrap(),
        };

        match get_response_to_request_threaded(ClientToLib3h::Bootstrap(bootstrap_data)) {
            GhostCallbackData::Response(Err(_)) => assert!(true),
            GhostCallbackData::Timeout => panic!("unexpected timeout"),
            r => panic!("unexpected response: {:?}", r),
        }
    }

    #[test]
    fn bootstrap_to_database_url_succeeds() {
        let engine = SimGhostActor::new(&LOCAL_ENDPOINT.to_string());

        let bootstrap_data = BootstrapData {
            space_address: Address::from(""),
            bootstrap_uri: Url::parse("http://fake_url").unwrap(),
        };

        match get_response_to_request(engine, ClientToLib3h::Bootstrap(bootstrap_data)) {
            GhostCallbackData::Response(Ok(ClientToLib3hResponse::BootstrapSuccess)) => {
                assert!(true)
            }
            r => panic!("unexpected response: {:?}", r),
        }
    }

    #[test]
    fn call_to_join_space_succeeds() {
        let engine = SimGhostActor::new(&LOCAL_ENDPOINT.to_string());
        let space_data = SpaceData {
            space_address: Address::from("space-123"),
            request_id: String::from("0"),
            agent_id: Address::from("an-agent"),
        };
        match get_response_to_request(engine, ClientToLib3h::JoinSpace(space_data)) {
            GhostCallbackData::Response(Ok(ClientToLib3hResponse::JoinSpaceResult)) => {
                assert!(true)
            }
            r => panic!("unexpected response: {:?}", r),
        }
    }
}
