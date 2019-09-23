//! provides worker that makes use of lib3h

use crate::connection::{
    net_connection::{NetHandler, NetWorker},
    NetResult,
};
use holochain_json_api::{error::JsonError, json::JsonString};
use lib3h_protocol::{
    protocol_client::Lib3hClientProtocol,
};
use log::debug;
use sim1h::dht::bbdht::dynamodb::client::{Client, client_from_endpoint};
use sim1h::workflow::to_client::connected::connected;
use url::Url;
use sim1h::workflow::from_client::join_space::join_space;

#[derive(Deserialize, Serialize, Clone, Debug, DefaultJson, PartialEq)]
pub struct Sim1hConfig{
    pub dynamo_url: String,
}

/// removed lifetime parameter because compiler says ghost engine needs lifetime that could live statically
#[allow(non_snake_case)]
pub struct Sim1hWorker {
    handler: NetHandler,
    dynamo_db_client: Client,
    inbox: Vec<Lib3hClientProtocol>,
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
    	Ok(Self { handler, dynamo_db_client, inbox: Vec::new() })
    }

    fn handle_client_message(&mut self, data: Lib3hClientProtocol) -> NetResult<Option<Lib3hClientProtocol>> {
        match data {
            // Success response to a request (any Command with an `request_id` field.)
            Lib3hClientProtocol::SuccessResult(_generic_result_data) => {},
            // Failure response to a request (any Command with an `request_id` field.)
            // Can also be a response to a mal-formed request.
            Lib3hClientProtocol::FailureResult(_generic_result_data) => {},
            // Connect to the specified multiaddr
            Lib3hClientProtocol::Connect(connect_data) => {
                let log_context = "Lib3hToClient::Connected";
                connected(&log_context, &self.dynamo_db_client, &connect_data);
                Ok(None)
            }

            // -- Space -- //
            // Order the p2p module to be part of the network of the specified space.
            Lib3hClientProtocol::JoinSpace(space_data) => {
                //let ClientToLib3h::JoinSpace(space_data)= ClientToLib3h::from(data);
                let log_context = "ClientToLib3h::JoinSpace";
                Ok(Lib3hClientProtocol::from(join_space(&log_context, &self.dbclient, &space_data)?))
            }
            // Order the p2p module to leave the network of the specified space.
            Lib3hClientProtocol::LeaveSpace(space_data) => {
                Ok(None)
            }

            // -- Direct Messaging -- //
            // Send a message directly to another agent on the network
            Lib3hClientProtocol::SendDirectMessage(DirectMessageData) => {
                Ok(None)
            },
            // Our response to a direct message from another agent.
            Lib3hClientProtocol::HandleSendDirectMessageResult(DirectMessageData) => {
                Ok(None)
            },
            // -- Entry -- //
            // Request an Entry from the dht network
            Lib3hClientProtocol::FetchEntry(FetchEntryData) => {
                Ok(None)
            },
            // Successful data response for a `HandleFetchEntryData` request
            Lib3hClientProtocol::HandleFetchEntryResult(FetchEntryResultData) => {
                Ok(None)
            },
            // Publish data to the dht.
            Lib3hClientProtocol::PublishEntry(ProvidedEntryData) => {
                Ok(None)
            },
            // Tell network module Core is holding this entry
            Lib3hClientProtocol::HoldEntry(ProvidedEntryData) => {
                Ok(None)
            },
            // Request some info / data from a Entry
            Lib3hClientProtocol::QueryEntry(QueryEntryData) => {
                Ok(None)
            },
            // Response to a `HandleQueryEntry` request
            Lib3hClientProtocol::HandleQueryEntryResult(QueryEntryResultData) => {
                Ok(None)
            },

            // -- Entry lists -- //
            Lib3hClientProtocol::HandleGetAuthoringEntryListResult(EntryListData) => {
                Ok(None)
            },
            Lib3hClientProtocol::HandleGetGossipingEntryListResult(EntryListData) => {
                Ok(None)
            },

            // -- N3h specific functinonality -- //
            Lib3hClientProtocol::Shutdown => {
                Ok(None)
            },
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
        println!("sim1h tick");
        let mut did_something = false;
        for data in self.inbox.drain(..) {
            self.handle_client_message(data);
            did_something = true;
        }
        Ok(did_something)
    }

    /// Set the advertise as worker's endpoint
    fn p2p_endpoint(&self) -> Option<url::Url> {
        Some(self.net_engine.advertise())
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
		data_types::*,
		protocol_server::Lib3hServerProtocol,
		protocol_client::Lib3hClientProtocol,
	};
	use url::Url;

	fn test_worker() -> (Sim1hWorker, crossbeam_channel::Receiver<NetResult<Lib3hServerProtocol>>) {
		let (s,r) = crossbeam_channel::unbounded();
		let handler = NetHandler::new(Box::new(move |message| {
			s.send(message).map_err(|e| e.into())
		}));
		(
            Sim1hWorker::new(
                handler,
                Sim1hConfig{
                    dynamo_url:"http://derp:8000".into()
                }
            ).unwrap(),
            r,
        )
	}

    #[test]
    fn call_to_boostrap_fails() {
    	let (mut worker, r) = test_worker();

    	let connect_data = ConnectData {
    		request_id: String::from("request-id-0"),
    		peer_uri: Url::parse("http://bs").unwrap(),
    		network_id: String::from("network-id"),
    	};
    	let message = Lib3hClientProtocol::Connect(connect_data);

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
