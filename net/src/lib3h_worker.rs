//! provides worker that makes use of lib3h

use crate::connection::{
    net_connection::{NetHandler, NetWorker},
    NetResult,
};
use lib3h::{
    dht::mirror_dht::MirrorDht,
    engine::{RealEngine, RealEngineConfig},
};

use lib3h_protocol::{network_engine::NetworkEngine, protocol_client::Lib3hClientProtocol};

/// A worker that makes use of lib3h / NetworkEngine.
/// It adapts the Worker interface with Lib3h's NetworkEngine's interface.
/// Handles `Protocol` and translates `JsonProtocol` to `Lib3hProtocol`.
/// TODO: currently uses MirrorDht, will need to expand workers to use different
/// generics.
#[allow(non_snake_case)]
pub struct Lib3hWorker<'a> {
    handler: NetHandler,
    net_engine: RealEngine<'a, MirrorDht>,
}


impl<'a> Lib3hWorker<'a> {
    pub fn advertise(self) -> url::Url {
        self.net_engine.advertise()
    }

}

/// Constructors
impl Lib3hWorker<'_> {
    /// Create a new websocket worker connected to the lib3h NetworkEngine
    pub fn with_wss_transport(handler: NetHandler, real_config: RealEngineConfig) -> NetResult<Self> {
        Ok(Lib3hWorker {
            handler,
            net_engine: RealEngine::new(
                Box::new(lib3h_sodium::SodiumCryptoSystem::new()),
                real_config,
                // TODO generate this automatically in the lib3h api
                "wss-agent",
                MirrorDht::new_with_config,
            )?,
        })
    }

    /// Create a new memory worker connected to the lib3h NetworkEngine
    pub fn with_memory_transport(handler: NetHandler, real_config: RealEngineConfig) -> NetResult<Self> {

        let net_engine = RealEngine::new_mock(
            Box::new(lib3h_sodium::SodiumCryptoSystem::new()),
            real_config.clone(),
            // TODO generate this automatically in the lib3h api
            format!("mem-agent-{}", snowflake::ProcessUniqueId::new()).as_str(),
            MirrorDht::new_with_config,
            )?;

        let worker = Lib3hWorker {
            handler,
            net_engine
        };

        Ok(worker)

    }
}

impl NetWorker for Lib3hWorker<'_> {
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
    // FIXME
}
