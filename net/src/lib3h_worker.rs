//! provides worker that makes use of lib3h

use crate::connection::{
    net_connection::{NetHandler, NetWorker},
    protocol::{PingData, Protocol},
    NetResult,
};
use holochain_core_types::json::JsonString;

use holochain_lib3h::real_engine::{RealEngine, RealEngineConfig};

use holochain_lib3h_protocol::network_engine::NetworkEngine;

/// A worker that makes use of lib3h / NetworkModule.
/// It adapts the Worker interface with Lib3h's NetworkModule's interface.
/// Handles `Protocol` and translates `JsonProtocol` to `Lib3hProtocol`.
#[allow(non_snake_case)]
pub struct Lib3hWorker {
    handler: NetHandler,
    can_send_P2pReady: bool,
    net_engine: RealEngine,
}

/// Constructors
impl Lib3hWorker {
    /// Create a new worker connected to the lib3h NetworkEngine
    pub fn new(handler: NetHandler, real_config: RealEngineConfig) -> NetResult<Self> {
        println!("RealEngineConfig: {:?}", real_config.clone());
        Ok(Lib3hWorker {
            handler,
            can_send_P2pReady: true,
            net_engine: RealEngine::new(real_config, "FIXME")?,
        })
    }
    /// Create a new worker connected to the lib3h NetworkEngine
    pub fn new_with_json_config(
        handler: NetHandler,
        backend_config: &JsonString,
    ) -> NetResult<Self> {
        let config: serde_json::Value = serde_json::from_str(backend_config.into())?;
        // manually deserialize RealEngineConfig
        let socket_type = match config["socketType"].as_str() {
            None => "ws",
            Some(st) => st,
        }
        .to_string();
        let bootstrap_nodes: Vec<String> = match config["bootstrapNodes"].as_array() {
            None => Vec::new(),
            Some(bs) => {
                // bs is &Vec<Value>, change it to Vec<String>
                let mut nodes: Vec<String> = Vec::new();
                for v in bs {
                    if let Some(s) = v.as_str() {
                        nodes.push(s.into());
                    }
                }
                nodes
            }
        };
        let work_dir = match config["workDir"].as_str() {
            None => String::new(),
            Some(wd) => wd.to_string(),
        };
        let log_level = match config["logLevel"].as_str() {
            None => 'i',
            Some(ll) => ll
                .chars()
                .next()
                .expect("logLevel setting should not be an empty string."),
        };
        let real_config = RealEngineConfig {
            socket_type,
            bootstrap_nodes,
            work_dir,
            log_level,
        };
        Lib3hWorker::new(handler, real_config)
    }
}

impl NetWorker for Lib3hWorker {
    /// We got a message from core
    /// -> forward it to the NetworkEngine
    fn receive(&mut self, data: Protocol) -> NetResult<()> {
        // Handle 'Shutdown' directly
        if data == Protocol::Shutdown {
            self.net_engine.terminate()?;
            (self.handler)(Ok(Protocol::Terminated))?;
            return Ok(());
        }
        // FIXME do translation
        // Post message
        if let Protocol::Lib3h(msg) = data {
            self.net_engine.post(msg.clone())?;
        }
        // Done
        Ok(())
    }

    /// Check for messages from our NetworkModule
    fn tick(&mut self) -> NetResult<bool> {
        // Send p2pReady on first tick
        if self.can_send_P2pReady {
            self.can_send_P2pReady = false;
            (self.handler)(Ok(Protocol::P2pReady))?;
        }
        // check for messages from our NetworkModule
        let (did_something, output) = self.net_engine.process()?;
        if did_something {
            for _msg in output {
                // FIXME translate Lib3hProtocol to Protocol
                (self.handler)(Ok(Protocol::Ping(PingData { sent: 4.2 })))?;
            }
        }
        Ok(did_something)
    }

    /// Stop the NetworkModule
    fn stop(self: Box<Self>) -> NetResult<()> {
        self.net_engine.stop()
    }

    /// Set the advertise as worker's endpoint
    fn endpoint(&self) -> Option<String> {
        Some(self.net_engine.advertise())
    }
}

/// Terminate on Drop
impl Drop for Lib3hWorker {
    fn drop(&mut self) {
        self.net_engine.terminate().ok();
    }
}

#[cfg(test)]
mod tests {
    // FIXME
}
