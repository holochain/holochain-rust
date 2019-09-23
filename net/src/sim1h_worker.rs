//! provides worker that makes use of lib3h

use crate::connection::{
    net_connection::{NetHandler, NetWorker},
    NetResult,
};
use holochain_json_api::{error::JsonError, json::JsonString};
use lib3h_protocol::{
    data_types::{GenericResultData, Opaque},
    protocol_client::Lib3hClientProtocol,
    protocol_server::Lib3hServerProtocol,
    Address,
};
use log::{debug, warn};
use sim1h::{
    dht::bbdht::dynamodb::client::{client_from_endpoint, Client},
    workflow::{
        from_client::{
            fetch_entry::fetch_entry, hold_entry::hold_entry, join_space::join_space,
            leave_space::leave_space, publish_entry::publish_entry, query_entry::query_entry,
            send_direct_message::send_direct_message,
        },
        to_client::{
            handle_fetch_entry::handle_fetch_entry,
            handle_get_authoring_entry_list::handle_get_authoring_entry_list,
            handle_get_gossiping_entry_list::handle_get_gossiping_entry_list,
            handle_query_entry::handle_query_entry,
            handle_send_direct_message::handle_send_direct_message,
        },
    },
};
use std::io::{self, Write};
use url::Url;

#[derive(Deserialize, Serialize, Clone, Debug, DefaultJson, PartialEq)]
pub struct Sim1hConfig {
    pub dynamo_url: String,
}

/// removed lifetime parameter because compiler says ghost engine needs lifetime that could live statically
#[allow(non_snake_case)]
pub struct Sim1hWorker {
    handler: NetHandler,
    dynamo_db_client: Client,
    inbox: Vec<Lib3hClientProtocol>,
    num_ticks: u32,
}

impl Sim1hWorker {
    pub fn advertise(self) -> url::Url {
        Url::parse("ws://example.com").unwrap()
    }
}

impl Sim1hWorker {
    /// Create a new websocket worker connected to the lib3h NetworkEngine
    pub fn new(handler: NetHandler, config: Sim1hConfig) -> NetResult<Self> {
        let dynamo_db_client = client_from_endpoint(config.dynamo_url);
        Ok(Self {
            handler,
            dynamo_db_client,
            inbox: Vec::new(),
            num_ticks: 0,
        })
    }

    fn handle_client_message(
        &mut self,
        data: Lib3hClientProtocol,
    ) -> NetResult<Lib3hServerProtocol> {
        match data {
            // Success response to a request (any Command with an `request_id` field.)
            Lib3hClientProtocol::SuccessResult(generic_result_data) => {
                Ok(Lib3hServerProtocol::FailureResult(generic_result_data))
            }
            // Failure response to a request (any Command with an `request_id` field.)
            // Can also be a response to a mal-formed request.
            Lib3hClientProtocol::FailureResult(generic_result_data) => {
                Ok(Lib3hServerProtocol::FailureResult(generic_result_data))
            }
            // Connect to the specified multiaddr
            Lib3hClientProtocol::Connect(connect_data) => {
                //let log_context = "Lib3hToClient::Connected";
                //connected(&log_context, &self.dynamo_db_client, &connect_data);
                Ok(Lib3hServerProtocol::FailureResult(GenericResultData {
                    request_id: connect_data.request_id,
                    space_address: Address::new(),
                    to_agent_id: Address::new(),
                    result_info: Opaque::new(),
                }))
            }

            // -- Space -- //
            // Order the p2p module to be part of the network of the specified space.
            Lib3hClientProtocol::JoinSpace(space_data) => {
                //let ClientToLib3h::JoinSpace(space_data)= ClientToLib3h::from(data);
                let log_context = "ClientToLib3h::JoinSpace";
                println!("handlingmessage {:?}", log_context);
                let _ = join_space(&log_context, &self.dynamo_db_client, &space_data)?;
                Ok(Lib3hServerProtocol::SuccessResult(GenericResultData {
                    request_id: space_data.request_id,
                    space_address: space_data.space_address,
                    to_agent_id: space_data.agent_id,
                    result_info: Opaque::new(),
                }))
            }
            // Order the p2p module to leave the network of the specified space.
            Lib3hClientProtocol::LeaveSpace(space_data) => {
                let log_context = "ClientToLib3h::LeaveSpace";
                println!("handlingmessage {:?}", log_context);
                let result = leave_space(&log_context, &self.dynamo_db_client, &space_data)?;
                Ok(result.into())
            }

            // -- Direct Messaging -- //
            // Send a message directly to another agent on the network
            Lib3hClientProtocol::SendDirectMessage(dm_data) => {
                let log_context = "ClientToLib3h::SendDirectMessage";
                println!("handlingmessage {:?}", log_context);
                let result = send_direct_message(&log_context, &self.dynamo_db_client, &dm_data)?;
                Ok(result.into())
            }
            // Our response to a direct message from another agent.
            Lib3hClientProtocol::HandleSendDirectMessageResult(dm_data) => {
                let log_context = "ClientToLib3h::HandleSendDirectMessageResult";
                println!("handlingmessage {:?}", log_context);
                handle_send_direct_message(&log_context, &self.dynamo_db_client, &dm_data);
                Ok(Lib3hServerProtocol::SuccessResult(GenericResultData {
                    request_id: dm_data.request_id,
                    space_address: dm_data.space_address,
                    to_agent_id: dm_data.to_agent_id,
                    result_info: Opaque::new(),
                }))
            }
            // -- Entry -- //
            // Request an Entry from the dht network
            Lib3hClientProtocol::FetchEntry(fetch_entry_data) => {
                let log_context = "ClientToLib3h::FetchEntry";
                println!("handlingmessage {:?}", log_context);
                let result = fetch_entry(&log_context, &self.dynamo_db_client, &fetch_entry_data)?;
                Ok(result.into())
            }
            // Successful data response for a `HandleFetchEntryData` request
            Lib3hClientProtocol::HandleFetchEntryResult(fetch_entry_result_data) => {
                let log_context = "ClientToLib3h::HandleFetchEntryResult";
                println!("handlingmessage {:?}", log_context);
                handle_fetch_entry(
                    &log_context,
                    &self.dynamo_db_client,
                    &fetch_entry_result_data,
                );
                Ok(Lib3hServerProtocol::SuccessResult(GenericResultData {
                    request_id: fetch_entry_result_data.request_id,
                    space_address: fetch_entry_result_data.space_address,
                    to_agent_id: fetch_entry_result_data.provider_agent_id,
                    result_info: Opaque::new(),
                }))
            }
            // Publish data to the dht.
            Lib3hClientProtocol::PublishEntry(provided_entry_data) => {
                let log_context = "ClientToLib3h::PublishEntry";
                println!("handlingmessage {:?}", log_context);
                publish_entry(&log_context, &self.dynamo_db_client, &provided_entry_data)?;
                Ok(Lib3hServerProtocol::SuccessResult(GenericResultData {
                    request_id: "".into(),
                    space_address: provided_entry_data.space_address,
                    to_agent_id: provided_entry_data.provider_agent_id,
                    result_info: Opaque::new(),
                }))
            }
            // Tell network module Core is holding this entry
            Lib3hClientProtocol::HoldEntry(provided_entry_data) => {
                let log_context = "ClientToLib3h::HoldEntry";
                println!("handlingmessage {:?}", log_context);
                hold_entry(&log_context, &self.dynamo_db_client, &provided_entry_data)?;
                Ok(Lib3hServerProtocol::SuccessResult(GenericResultData {
                    request_id: "".into(),
                    space_address: provided_entry_data.space_address,
                    to_agent_id: provided_entry_data.provider_agent_id,
                    result_info: Opaque::new(),
                }))
            }
            // Request some info / data from a Entry
            Lib3hClientProtocol::QueryEntry(query_entry_data) => {
                let log_context = "ClientToLib3h::QueryEntry";
                println!("handlingmessage {:?}", log_context);
                let result = query_entry(&log_context, &self.dynamo_db_client, &query_entry_data)?;
                Ok(result.into())
            }
            // Response to a `HandleQueryEntry` request
            Lib3hClientProtocol::HandleQueryEntryResult(query_entry_result_data) => {
                let log_context = "ClientToLib3h::HandleQueryEntryResult";
                println!("handlingmessage {:?}", log_context);
                handle_query_entry(
                    &log_context,
                    &self.dynamo_db_client,
                    &query_entry_result_data,
                );
                Ok(Lib3hServerProtocol::SuccessResult(GenericResultData {
                    request_id: query_entry_result_data.request_id,
                    space_address: query_entry_result_data.space_address,
                    to_agent_id: query_entry_result_data.requester_agent_id,
                    result_info: Opaque::new(),
                }))
            }

            // -- Entry lists -- //
            Lib3hClientProtocol::HandleGetAuthoringEntryListResult(entry_list_data) => {
                let log_context = "ClientToLib3h::HandleGetAuthoringEntryListResult";
                println!("handlingmessage {:?}", log_context);
                handle_get_authoring_entry_list(
                    &log_context,
                    &self.dynamo_db_client,
                    &entry_list_data,
                );
                Ok(Lib3hServerProtocol::SuccessResult(GenericResultData {
                    request_id: entry_list_data.request_id,
                    space_address: entry_list_data.space_address,
                    to_agent_id: entry_list_data.provider_agent_id,
                    result_info: Opaque::new(),
                }))
            }
            Lib3hClientProtocol::HandleGetGossipingEntryListResult(entry_list_data) => {
                let log_context = "ClientToLib3h::HandleGetGossipingEntryListResult";
                println!("handlingmessage {:?}", log_context);
                handle_get_gossiping_entry_list(
                    &log_context,
                    &self.dynamo_db_client,
                    &entry_list_data,
                );
                Ok(Lib3hServerProtocol::SuccessResult(GenericResultData {
                    request_id: entry_list_data.request_id,
                    space_address: entry_list_data.space_address,
                    to_agent_id: entry_list_data.provider_agent_id,
                    result_info: Opaque::new(),
                }))
            }

            // -- N3h specific functinonality -- //
            Lib3hClientProtocol::Shutdown => {
                Ok(Lib3hServerProtocol::FailureResult(GenericResultData {
                    request_id: "".into(),
                    space_address: Address::new(),
                    to_agent_id: Address::new(),
                    result_info: Opaque::new(),
                }))
            }
        }
    }
}

// TODO: DRY this up as it is basically the same as the lib3h engine
impl NetWorker for Sim1hWorker {
    /// We got a message from core
    /// -> forward it to the NetworkEngine
    fn receive(&mut self, data: Lib3hClientProtocol) -> NetResult<()> {
        debug!(">>NET>> {:?}", data);
        self.inbox.push(data);
        Ok(())
    }

    /// Check for messages from our NetworkEngine
    fn tick(&mut self) -> NetResult<bool> {
        self.num_ticks += 1;
        if self.num_ticks % 10 == 0 {
            print!("10ticks ");
        }
        if self.num_ticks % 100 == 0 {
            io::stdout().flush()?;
        }
        let mut did_something = false;
        let messages = self.inbox.drain(..).collect::<Vec<_>>();
        for data in messages {
            match self.handle_client_message(data) {
                Ok(response) => {
                    if let Err(error) = self.handler.handle(Ok(response)) {
                        warn!("Error returned from network handler in Sim1h: {:?}", error);
                    }
                }
                Err(error) => {
                    warn!("Error handling client message in Sim1hWorker: {:?}", error);
                }
            }
            did_something = true;
        }
        Ok(did_something)
    }

    /// Set the advertise as worker's endpoint
    fn p2p_endpoint(&self) -> Option<url::Url> {
        None
    }

    /// Set the advertise as worker's endpoint
    fn endpoint(&self) -> Option<String> {
        Some("".into())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use lib3h_protocol::{
        data_types::*, protocol_client::Lib3hClientProtocol, protocol_server::Lib3hServerProtocol,
    };
    //use url::Url;

    fn test_worker() -> (
        Sim1hWorker,
        crossbeam_channel::Receiver<NetResult<Lib3hServerProtocol>>,
    ) {
        let (s, r) = crossbeam_channel::unbounded();
        let handler = NetHandler::new(Box::new(move |message| {
            s.send(message).map_err(|e| e.into())
        }));
        (
            Sim1hWorker::new(
                handler,
                Sim1hConfig {
                    dynamo_url: "http://localhost:8000".into(),
                },
            )
            .unwrap(),
            r,
        )
    }

    #[test]
    fn call_to_boostrap_fails() {
        let (mut worker, r) = test_worker();

        /*
        let connect_data = ConnectData {
            request_id: String::from("request-id-0"),
            peer_uri: Url::parse("http://bs").unwrap(),
            network_id: String::from("network-id"),
        };
        let message = Lib3hClientProtocol::Connect(connect_data);
        */

        let space_data = SpaceData {
            request_id: String::from("request-id-0"),
            space_address: Address::from("test-space-address"),
            agent_id: Address::from("test-agent-id"),
        };
        let message = Lib3hClientProtocol::JoinSpace(space_data);

        // send the bootstrap message
        worker.receive(message).expect("could not send message");

        // tick a few times..
        for _ in 0..5 {
            worker.tick().ok();
        }

        // see if anything came down the channel
        let response = r.recv().expect("could not get response");

        println!("{:?}", response);
    }
}
