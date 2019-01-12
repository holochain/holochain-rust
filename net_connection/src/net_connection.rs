use super::NetResult;
use crate::protocol::Protocol;

/// closure for processing a Network Protocol message
pub type NetHandler = Box<FnMut(NetResult<Protocol>) -> NetResult<()> + Send>;

/// closure for doing any clean up at shutdown of a NetWorker
pub type NetShutdown = Option<Box<::std::boxed::FnBox() + Send>>;

///  Trait for sending Network Protocol messages
pub trait NetSend {
    fn send(&mut self, data: Protocol) -> NetResult<()>;
}


/// Trait for receiving Network Protocol messages
/// represents a worker that handles protocol messages
pub trait NetReceive {
    /// The receiving method when something has called `send()` to send this worker a message
    fn receive(&mut self, _data: Protocol) -> NetResult<()> {
        Ok(())
    }

    /// perform any upkeep
    /// return `false` if no particular upkeep has been processed
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

// TODO trait NetTicker/NetWorker with tick() and stop()

/// closure for instantiating a NetReceive from a NetHandler
pub type NetReceiverFactory =
    Box<::std::boxed::FnBox(NetHandler) -> NetResult<Box<NetReceive>> + Send>;
