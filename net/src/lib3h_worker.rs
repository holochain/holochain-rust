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

/// Common interface for all types of network modules used by the Lib3hWorker
/// TODO: move to lib3h crate
pub trait NetworkModule {
    /// Start network communications
    fn run(&mut self) -> NetResult<()>;
    /// Stop network communications
    fn stop(&mut self) -> NetResult<()>;
    /// Terminate module. Perform some cleanup.
    fn terminate(&self) -> NetResult<()>;
    /// Handle some data message sent by the local Client
    fn serve(&mut self, data: Protocol) -> NetResult<()>;
    /// Get qualified transport address
    fn advertise(&self) -> String;
}

/// 'Real mode' implementation of Lib3h as a NetworkModule
/// TODO: move to lib3h crate
struct Lib3hMain {
    config: serde_json::Value,
    tx: mpsc::Receiver<Protocol>,
}

impl Lib3hMain {
    pub fn new(config: serde_json::Value, tx: mpsc::Receiver<Protocol>) -> NetResult<Self> {
        Ok(Lib3hMain {
            config,
            tx,
        })
    }
}
impl NetworkModule for Lib3hMain {
    fn run(&mut self) -> NetResult<()> {
        Ok(())
    }
    fn stop(&mut self) -> NetResult<()> {
        Ok(())
    }
    fn terminate(&mut self) -> NetResult<()> {
        Ok(())
    }
    fn advertise(&self) -> String {
        "FIXME"
    }

    /// process a message sent by our local Client
    fn serve(&mut self, data: Protocol) -> NetResult<()> {
        self.log
            .d(&format!(">>>> '{}' recv: {:?}", self.name.clone(), data));
        // serve only JsonProtocol
        let maybe_json_msg = JsonProtocol::try_from(&data);
        if maybe_json_msg.is_err() {
            return Ok(());
        };
        // Note: use same order as the enum
        match maybe_json_msg.as_ref().unwrap() {
            JsonProtocol::SuccessResult(msg) => {
                // FIXME
            }
            JsonProtocol::FailureResult(msg) => {
                // FIXME
            }
            JsonProtocol::TrackDna(msg) => {
                // FIXME
            }
            JsonProtocol::UntrackDna(msg) => {
                // FIXME
            }
            JsonProtocol::SendMessage(msg) => {
                // FIXME
            }
            JsonProtocol::HandleSendMessageResult(msg) => {
                // FIXME
            }
            JsonProtocol::FetchEntry(msg) => {
                // FIXME
            }
            JsonProtocol::HandleFetchEntryResult(msg) => {
                // FIXME
            }
            JsonProtocol::PublishEntry(msg) => {
                // FIXME
            }
            JsonProtocol::FetchMeta(msg) => {
                // FIXME
            }
            JsonProtocol::HandleFetchMetaResult(msg) => {
                // FIXME
            }
            JsonProtocol::PublishMeta(msg) => {
                // FIXME
            }
            // Our request for the publish_list has returned
            JsonProtocol::HandleGetPublishingEntryListResult(msg) => {
                // FIXME
            }
            // Our request for the hold_list has returned
            JsonProtocol::HandleGetHoldingEntryListResult(msg) => {
                // FIXME
            }
            // Our request for the publish_meta_list has returned
            JsonProtocol::HandleGetPublishingMetaListResult(msg) => {
                // FIXME
            }
            // Our request for the hold_meta_list has returned
            JsonProtocol::HandleGetHoldingMetaListResult(msg) => {
                // FIXME
            }
            _ => {
                panic!("unexpected {:?}", &maybe_json_msg));
            }
        }
        Ok(())
    }
}

/// A worker that makes use of lib3h / NetworkModule
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
    /// we got a message from holochain core
    /// forward to NetworkModule
    fn receive(&mut self, data: Protocol) -> NetResult<()> {
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

    /// check for messages from our NetworkModule
    fn tick(&mut self) -> NetResult<bool> {
        // Send p2pready on first tick
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

    /// stop the net worker
    fn stop(self: Box<Self>) -> NetResult<()> {
        self.net_module.stop()
    }

    /// Set server's name as worker's endpoint
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
    /// FIXME
}
