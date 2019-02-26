use super::{protocol::Protocol, NetResult};

/// closure for processing a Protocol message received from the network
pub type NetHandler = Box<FnMut(NetResult<Protocol>) -> NetResult<()> + Send>;

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
