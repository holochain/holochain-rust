//! provides worker that makes use of lib3h

use crate::connection::{
    json_protocol::JsonProtocol,
    net_connection::{NetHandler, NetWorker},
    protocol::Protocol,
    NetResult,
};
use holochain_core_types::{cas::content::Address, json::JsonString};
use std::{
    collections::{hash_map::Entry, HashMap},
    convert::TryFrom,
    sync::{mpsc, Mutex},
};

/// A worker that makes use of lib3h / NetworkModule.
/// It adapts the Worker interface with Lib3h's NetworkModule's interface.
/// Handles `Protocol` and translates `JsonProtocol` to `Lib3hProtocol`.
#[allow(non_snake_case)]
pub struct Lib3hWorker {
    handler: NetHandler,
    rx: mpsc::Receiver<Protocol>,
    can_send_P2pReady: bool,
    net_module: Lib3hMain,
}

/// Constructors
impl Lib3hWorker {
    /// create a new worker connected to a lib3hMain
    pub fn new(handler: NetHandler, backend_config: &JsonString) -> NetResult<Self> {
        let config: serde_json::Value = serde_json::from_str(backend_config.into())?;
        let (tx, rx) = mpsc::channel();
        Ok(Lib3hWorker {
            handler,
            rx,
            can_send_P2pReady: true,
            net_module: Lib3hMain::new(config, tx)?,
        })
    }
}

impl NetWorker for Lib3hWorker {
    /// We got a message from holochain core
    /// -> forward it to the NetworkModule
    fn receive(&mut self, data: Protocol) -> NetResult<()> {
        // Handle 'Shutdown' directly
        if data == Protocol::Shutdown {
            self.net_module.terminate()?;
            (self.handler)(Ok(Protocol::Terminated))?;
            return Ok(());
        }
        // Serve data message
        self.net_module.serve(data.clone())?;
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
        let mut did_something = false;
        if let Ok(data) = receiver.try_recv() {
            did_something = true;
            (self.handler)(Ok(data))?;
        }
        Ok(did_something)
    }

    /// Stop the NetworkModule
    fn stop(self: Box<Self>) -> NetResult<()> {
        self.net_module.stop()
    }

    /// Set the advertise as worker's endpoint
    fn endpoint(&self) -> Option<String> {
        Some(self.net_module.advertise())
    }
}

/// Terminate on Drop
impl Drop for Lib3hWorker {
    fn drop(&mut self) {
        self.net_module.terminate().ok();
    }
}

#[cfg(test)]
mod tests {
    // FIXME
}
