use holochain_walkman_types::{Cassette, WalkmanEvent, WalkmanLogItem, WalkmanSim2hEvent};
use sim2h_client::Sim2hClient;
use std::collections::{hash_map::Entry, HashMap};
use url2::Url2;

#[derive(Default)]
pub struct Sim2hCassettePlayer {
    clients: HashMap<String, Sim2hClient>,
}

impl Sim2hCassettePlayer {
    pub fn playback(&mut self, cassette: Cassette) {
        for event in cassette.events() {
            match event {
                WalkmanLogItem {
                    time: _,
                    event: WalkmanEvent::Sim2hEvent(event),
                } => match event {
                    WalkmanSim2hEvent::Connect(url) => match self.clients.entry(url.clone()) {
                        Entry::Vacant(e) => {
                            e.insert(
                                Sim2hClient::new(&Url2::parse(url))
                                    .expect("Couldn't create sim2h client"),
                            );
                        }
                        Entry::Occupied(_) => {
                            panic!(format!("Tried to connect from url twice: {}", url))
                        }
                    },
                    WalkmanSim2hEvent::Disconnect(url) => match self.clients.entry(url.clone()) {
                        Entry::Occupied(e) => {
                            e.remove_entry();
                        }
                        Entry::Vacant(_) => panic!(format!(
                            "Tried to disconnect from url without being connected: {}",
                            url
                        )),
                    },
                    WalkmanSim2hEvent::Message(url, message_str) => self
                        .clients
                        .get(url)
                        .map(|_client| {
                            println!("TODO, send msg: {:?}", message_str);
                        })
                        .unwrap_or_else(|| {
                            panic!("Trying to send message without a client connection")
                        }),
                },
            }
        }
    }
}
