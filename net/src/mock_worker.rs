//! provides fake in-memory p2p worker for use in scenario testing

use holochain_core_types::json::JsonString;
use holochain_net_connection::{
    net_connection::{NetHandler, NetReceive},
    protocol::Protocol,
    protocol_wrapper::{
        ProtocolWrapper,
    },
    NetResult,
};

use std::{
    convert::TryFrom,
    sync::mpsc,
};

use mock_singleton::get_mock_singleton;

use holochain_net_ipc::util::get_millis;

/// a p2p worker for mocking in-memory scenario tests
pub struct MockWorker {
    handler: NetHandler,
    receivers: Vec<mpsc::Receiver<Protocol>>,
    _network_name: String, // TODO use this to uniquify MockSystem
    last_state_query_millis: f64,
    last_known_state: String,
}

impl NetReceive for MockWorker {
    /// Stop the net worker: Nothing to do for the mock
    fn stop(self: Box<Self>) -> NetResult<()> {
        Ok(())
    }

    /// we got a message from holochain core
    /// forward to our mock singleton
    fn receive(&mut self, data: Protocol) -> NetResult<()> {

        println!("MockWorker::receive(): {:?}", data);

        let mut mock_singleton = get_mock_singleton()?;

        // Special case: Register tracking appropriately
        if let Ok(msg) = ProtocolWrapper::try_from(&data) {
            if let ProtocolWrapper::TrackDna(track_dna_msg) = msg {
                let (tx, rx) = mpsc::channel();
                self.receivers.push(rx);
                mock_singleton.register(&track_dna_msg.dna_address, &track_dna_msg.agent_id, tx)?;
                return Ok(());
            }
        }

        mock_singleton.handle(data)?;
        Ok(())
    }

    /// check for messages on all trackings (All Senders should be from a MockSingleton)
    fn tick(&mut self) -> NetResult<bool> {
        let mut did_something = false;

//        if &self.last_known_state != "ready" {
//            self.priv_check_init()?;
//        }

        // Handle incoming data on each Receiver
        for rx in self.receivers.iter_mut() {
            if let Ok(data) = rx.try_recv() {
                did_something = true;
                (self.handler)(Ok(data))?;
            }
        }

        Ok(did_something)
    }
}

impl MockWorker {
    /// Constructor
    pub fn new(handler: NetHandler, network_config: &JsonString) -> Self {
        let config: serde_json::Value = serde_json::from_str(network_config.into()).expect("Invalid network config for MockWorker");
        let _network_name = config["networkName"]
            .as_str()
            .unwrap_or("(unnamed)")
            .to_string();

        MockWorker {
            handler,
            receivers: Vec::new(),
            _network_name,
            last_state_query_millis: 0.0_f64,
            last_known_state: "undefined".to_string(),
        }
    }

    /// send a ping twice per second
    fn priv_check_init(&mut self) -> NetResult<()> {
        let now = get_millis();

        if now - self.last_state_query_millis > 500.0 {
            let mut mock_singleton = get_mock_singleton()?;
            mock_singleton.handle(ProtocolWrapper::RequestState.into())?;
            self.last_state_query_millis = now;
        }

        Ok(())
    }
}
