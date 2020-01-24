use holochain_walkman_types::{Cassette, WalkmanEvent, WalkmanLogItem, WalkmanSim2hEvent};
use in_stream::InStream;
use lib3h_protocol::{data_types::Opaque, protocol::*};
use sim2h::{crypto::SignedWireMessage, wire_message::WireMessage};
use sim2h_client::Sim2hClient;
use std::{
    collections::{hash_map::Entry, HashMap},
    convert::TryInto,
};
use url2::Url2;

#[derive(Default)]
pub struct Sim2hCassettePlayer {}

impl Sim2hCassettePlayer {
    pub fn playback(sim2h_url: &Url2, cassette: Cassette) {
        let mut clients: HashMap<String, Sim2hClient> = HashMap::new();
        for event in cassette.events() {
            match event {
                WalkmanLogItem {
                    time: _,
                    event: WalkmanEvent::Sim2hEvent(event),
                } => match event {
                    WalkmanSim2hEvent::Connect(client_url) => match clients
                        .entry(client_url.clone())
                    {
                        Entry::Vacant(e) => {
                            e.insert(
                                Sim2hClient::new(sim2h_url).expect("Couldn't create sim2h client"),
                            );
                        }
                        Entry::Occupied(_) => {
                            panic!(format!("Tried to connect from url twice: {}", client_url))
                        }
                    },
                    WalkmanSim2hEvent::Disconnect(client_url) => {
                        match clients.entry(client_url.clone()) {
                            Entry::Occupied(e) => {
                                e.remove_entry();
                            }
                            Entry::Vacant(_) => panic!(format!(
                                "Tried to disconnect from url without being connected: {}",
                                client_url
                            )),
                        }
                    }
                    WalkmanSim2hEvent::Message(client_url, message_str) => clients
                        .get_mut(client_url)
                        .map(|client| {
                            // The Sim2hClient was created with a random keypair, but we are going to bypass that agent here
                            // and directly send a saved signed message from a different prior Agent
                            let msg: SignedWireMessage = deserialize_message_data(message_str);
                            let wire_msg: WireMessage = get_wire_message(&msg);
                            println!("Playback WireMessage from {} : {:?}", client_url, wire_msg);
                            let to_send: Opaque = msg.into();
                            client
                                .connection()
                                .write(to_send.as_bytes().into())
                                .unwrap();

                            if let WireMessage::ClientToLib3h(ClientToLib3h::JoinSpace(_)) = wire_msg {
                                // We need to wait for the JoinSpace to complete on the sim2h side,
                                // but JoinSpaceResult is never sent by sim2h, so we do this hacky waiting
                                println!("Awaiting Lib3hToClient::HandleGetGossipingEntryListResult after JoinSpace");
                                let _ = client.await_msg(|msg| {
                                    if let WireMessage::Lib3hToClient(Lib3hToClient::HandleGetGossipingEntryList(_)) = msg {
                                        true
                                    } else {
                                        false
                                    }
                                });
                                println!("Now waiting 100ms because we don't know when the Join is actually done...");
                                std::thread::sleep(std::time::Duration::from_millis(100));
                            }
                        })
                        .unwrap_or_else(|| {
                            panic!("Trying to send message without a client connection")
                        }),
                },
            }
        }
    }
}

pub fn deserialize_message_data(data: &str) -> SignedWireMessage {
    serde_json::from_str(data).expect("Couldn't parse serialized SignedWireMessage")
}

pub fn get_wire_message(signed: &SignedWireMessage) -> WireMessage {
    signed.clone().payload.try_into().unwrap()
}
