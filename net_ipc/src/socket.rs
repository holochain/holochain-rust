//! This module is a thin wrapper around a ZMQ socket
//! It allows us to easily mock it out for unit tests
//! as well as manage context with lazy_static!

use context;
use errors::*;
use zmq;

/// trait that allows zmq socket abstraction
pub trait IpcSocket {
    /// clean up the context
    fn destroy_context() -> Result<()>;

    /// create a new socket
    fn new() -> Result<Box<Self>>
    where
        Self: Sized;

    /// close an existing socket
    fn close(self: Box<Self>) -> Result<()>;

    /// connect the socket to a remote end
    fn connect(&mut self, endpoint: &str) -> Result<()>;

    /// see if we have any messages waiting
    fn poll(&mut self, millis: i64) -> Result<bool>;

    /// if we DO have messages, fetch them
    fn recv(&mut self) -> Result<Vec<Vec<u8>>>;

    /// send data to the remote end of the socket
    fn send(&mut self, data: &[&[u8]]) -> Result<()>;
}

/// this is the concrete ZMQ implementation of the IpcSocket trait
pub struct ZmqIpcSocket {
    socket: zmq::Socket,
}

impl IpcSocket for ZmqIpcSocket {
    fn destroy_context() -> Result<()> {
        context::destroy()?;
        Ok(())
    }

    fn new() -> Result<Box<Self>> {
        Ok(Box::new(Self {
            socket: context::socket(zmq::ROUTER)?,
        }))
    }

    #[allow(unknown_lints)]
    #[allow(boxed_local)] // (neonphog) required for sizing on trait IpcSocket
    fn close(self: Box<Self>) -> Result<()> {
        drop(self.socket);
        Ok(())
    }

    fn connect(&mut self, endpoint: &str) -> Result<()> {
        self.socket.connect(endpoint)?;
        Ok(())
    }

    fn poll(&mut self, millis: i64) -> Result<bool> {
        Ok(self.socket.poll(zmq::POLLIN, millis)? != 0)
    }

    fn recv(&mut self) -> Result<Vec<Vec<u8>>> {
        Ok(self.socket.recv_multipart(0)?)
    }

    fn send(&mut self, data: &[&[u8]]) -> Result<()> {
        self.socket.send_multipart(data, 0)?;
        Ok(())
    }
}

#[cfg(test)]
/// This is a concrete implementation of the IpcSocket trait for use in testing
pub struct MockIpcSocket {
    resp_queue: Vec<Vec<Vec<u8>>>,
    sent_queue: Vec<Vec<Vec<u8>>>,
}

#[cfg(test)]
impl MockIpcSocket {
    pub fn inject_response(&mut self, data: Vec<Vec<u8>>) {
        self.resp_queue.push(data);
    }

    pub fn sent_count(&self) -> usize {
        self.sent_queue.len()
    }

    pub fn next_sent(&mut self) -> Vec<Vec<u8>> {
        self.sent_queue.remove(0)
    }
}

#[cfg(test)]
impl IpcSocket for MockIpcSocket {
    fn destroy_context() -> Result<()> {
        Ok(())
    }

    fn new() -> Result<Box<Self>> {
        Ok(Box::new(Self {
            resp_queue: Vec::new(),
            sent_queue: Vec::new(),
        }))
    }

    fn close(self: Box<Self>) -> Result<()> {
        Ok(())
    }

    fn connect(&mut self, _endpoint: &str) -> Result<()> {
        Ok(())
    }

    fn poll(&mut self, _millis: i64) -> Result<bool> {
        Ok(!self.resp_queue.is_empty())
    }

    fn recv(&mut self) -> Result<Vec<Vec<u8>>> {
        Ok(self.resp_queue.remove(0))
    }

    fn send(&mut self, data: &[&[u8]]) -> Result<()> {
        let mut tmp: Vec<Vec<u8>> = Vec::new();
        for item in data {
            tmp.push(item.to_vec());
        }
        self.sent_queue.push(tmp);
        Ok(())
    }
}
