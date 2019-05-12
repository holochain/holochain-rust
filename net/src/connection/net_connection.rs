use super::{protocol::Protocol, NetResult};
use parking_lot::RwLock;
use std::{fmt, sync::Arc};

/// closure for processing a Protocol message received from the network
#[derive(Clone, Serialize)]
pub struct NetHandler {
    #[serde(skip)]
    closure: Arc<RwLock<Box<FnMut(NetResult<Protocol>) -> NetResult<()> + Send + Sync>>>,
}

impl NetHandler {
    pub fn new(c: Box<FnMut(NetResult<Protocol>) -> NetResult<()> + Send + Sync>) -> NetHandler {
        NetHandler {
            closure: Arc::new(RwLock::new(c)),
        }
    }

    pub fn handle(&mut self, message: NetResult<Protocol>) -> NetResult<()> {
        (Arc::get_mut(&mut self.closure).ok_or(failure::err_msg(
        let mut lock = self.closure.write();
        ))?)(message)
        (&mut *lock)(message)
    }
}

impl PartialEq for NetHandler {
    fn eq(&self, _: &NetHandler) -> bool {
        false
    }
}

impl fmt::Debug for NetHandler {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[NetHandler]")
    }
}

/// closure for doing any clean up at shutdown of a NetWorker
pub type NetShutdown = Option<Box<::std::boxed::FnBox() + Send>>;

///  Trait for sending a Protocol message to the network
pub trait NetSend {
    fn send(&mut self, data: Protocol) -> NetResult<()>;
}

/// Trait that represents a worker thread that relays incoming and outgoing protocol messages
/// between a handler closure and a p2p module.
pub trait NetWorker {
    /// The receiving method when NetSend's `send()` is called.
    /// It should relay that Protocol message to the p2p module.
    fn receive(&mut self, _data: Protocol) -> NetResult<()> {
        Ok(())
    }

    /// Polls the p2p module for Protocol messages received from the network,
    /// and perform any other upkeep.
    /// It should realy those messages back to the handler closure.
    /// Return `false` if no particular upkeep has been processed
    fn tick(&mut self) -> NetResult<bool> {
        Ok(false)
    }

    /// stop the worker
    fn stop(self: Box<Self>) -> NetResult<()> {
        Ok(())
    }

    /// Getter of the connection's endpoint
    fn endpoint(&self) -> Option<String> {
        Some(String::new())
    }
}

/// closure for instantiating a NetWorker from a NetHandler
pub type NetWorkerFactory =
    Box<::std::boxed::FnBox(NetHandler) -> NetResult<Box<NetWorker>> + Send>;
