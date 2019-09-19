//! provides worker that makes use of lib3h

use crate::connection::{
    net_connection::{NetHandler, NetWorker},
    NetResult,
};
use lib3h::{
    engine::{ghost_engine_wrapper::LegacyLib3h},
    error::Lib3hError
};
use sim1h::ghost_actor::SimGhostActor;

use lib3h_protocol::protocol_client::Lib3hClientProtocol;

/// removed lifetime parameter because compiler says ghost engine needs lifetime that could live statically
#[allow(non_snake_case)]
pub struct Sim1hWorker {
    handler: NetHandler,
    net_engine : LegacyLib3h<SimGhostActor, Lib3hError>
}


impl Sim1hWorker {
    pub fn advertise(self) -> url::Url {
        self.net_engine.advertise()
    }

}

impl Sim1hWorker {
    /// Create a new websocket worker connected to the lib3h NetworkEngine
    pub fn new(handler: NetHandler) -> NetResult<Self> {
    	// TODO: Don't be stupd and actualy take this as a param
    	let ghost_engine = SimGhostActor::new(&"http://derp:8000".into());
    	Ok(Self {
    		handler,
    		net_engine: LegacyLib3h::new("core", ghost_engine),
    	})
    }
}


// TODO: DRY this up as it is basically the same as the lib3h engine
impl NetWorker for Sim1hWorker {
    /// We got a message from core
    /// -> forward it to the NetworkEngine
    fn receive(&mut self, data: Lib3hClientProtocol) -> NetResult<()> {
        self.net_engine.post(data.clone())?;
        // Done
        Ok(())
    }

    /// Check for messages from our NetworkEngine
    fn tick(&mut self) -> NetResult<bool> {
        // Tick the NetworkEngine and check for incoming protocol messages.
        let (did_something, output) = self.net_engine.process()?;
        if did_something {
            for msg in output {
                self.handler.handle(Ok(msg))?;
            }
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
		(Sim1hWorker::new(handler).unwrap(), r)
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
