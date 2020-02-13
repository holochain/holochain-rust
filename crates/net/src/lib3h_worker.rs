//! provides worker that makes use of lib3h

use crate::{
    connection::{
        net_connection::{NetHandler, NetWorker},
        NetResult,
    },
    p2p_network::Lib3hClientProtocolWrapped,
    NEW_RELIC_LICENSE_KEY,
};
use lib3h::{
    dht::mirror_dht::MirrorDht,
    engine::{ghost_engine_wrapper::LegacyLib3h, EngineConfig, GhostEngine},
    error::Lib3hError,
};

use holochain_tracing::Span;

/// A worker that makes use of lib3h / NetworkEngine.
/// It adapts the Worker interface with Lib3h's NetworkEngine's interface.
/// Handles `Protocol` and translates `JsonProtocol` to `Lib3hProtocol`.
/// TODO: currently uses MirrorDht, will need to expand workers to use different
/// generics.
///
/// removed lifetime parameter because compiler says ghost engine needs lifetime that could live statically
#[allow(non_snake_case)]
pub struct Lib3hWorker {
    handler: NetHandler,
    net_engine: LegacyLib3h<GhostEngine<'static>, Lib3hError>,
}

impl Lib3hWorker {
    pub fn advertise(self) -> url::Url {
        self.net_engine.advertise().into()
    }
}

/// Constructors
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_NET)]
impl Lib3hWorker {
    /// Create a new websocket worker connected to the lib3h NetworkEngine
    pub fn with_wss_transport(handler: NetHandler, real_config: EngineConfig) -> NetResult<Self> {
        Ok(Lib3hWorker {
            handler,
            net_engine: LegacyLib3h::new(
                "core",
                GhostEngine::new(
                    Span::fixme(),
                    Box::new(lib3h_sodium::SodiumCryptoSystem::new()),
                    real_config,
                    // TODO generate this automatically in the lib3h api
                    "wss-agent",
                    MirrorDht::new_with_config,
                )?,
            ),
        })
    }

    /// Create a new memory worker connected to the lib3h NetworkEngine
    pub fn with_memory_transport(
        handler: NetHandler,
        real_config: EngineConfig,
    ) -> NetResult<Self> {
        let ghost_engine = GhostEngine::new(
            Span::fixme(),
            Box::new(lib3h_sodium::SodiumCryptoSystem::new()),
            real_config.clone(),
            // TODO generate this automatically in the lib3h api
            format!("mem-agent-{}", snowflake::ProcessUniqueId::new()).as_str(),
            MirrorDht::new_with_config,
        )?;
        let net_engine = LegacyLib3h::new("core", ghost_engine);
        let worker = Lib3hWorker {
            handler,
            net_engine,
        };

        Ok(worker)
    }
}

impl NetWorker for Lib3hWorker {
    /// We got a message from core
    /// -> forward it to the NetworkEngine
    fn receive(&mut self, data: Lib3hClientProtocolWrapped) -> NetResult<()> {
        self.net_engine.post(data.data.clone())?;
        // Done
        Ok(())
    }

    /// Check for messages from our NetworkEngine
    fn tick(&mut self) -> NetResult<bool> {
        let span = ht::with_top_or_null(|s| s.child("pre-send"));
        // Tick the NetworkEngine and check for incoming protocol messages.
        let (did_something, output) = self.net_engine.process()?;
        if did_something {
            for msg in output {
                self.handler
                    .handle(Ok(span.follower("inner").wrap(msg).into()))?;
            }
        }
        Ok(did_something)
    }

    /// Set the advertise as worker's endpoint
    fn p2p_endpoint(&self) -> Option<url::Url> {
        Some(self.net_engine.advertise().into())
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
