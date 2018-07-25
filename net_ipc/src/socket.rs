use context;
use errors::*;
use zmq;

pub trait IpcSocket {
    fn destroy_context() -> Result<()>;
    fn new() -> Result<Box<Self>>
    where
        Self: Sized;
    fn close(self: Box<Self>) -> Result<()>;
    fn connect(&mut self, endpoint: &str) -> Result<()>;
    fn poll(&mut self, millis: i64) -> Result<bool>;
    fn recv(&mut self) -> Result<Vec<Vec<u8>>>;
    fn send(&mut self, data: &[&[u8]]) -> Result<()>;
}

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
pub struct MockIpcSocket {
    resp_queue: Vec<Vec<Vec<u8>>>,
}

#[cfg(test)]
impl MockIpcSocket {
    pub fn inject_response(&mut self, data: Vec<Vec<u8>>) {
        self.resp_queue.push(data);
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

    fn send(&mut self, _data: &[&[u8]]) -> Result<()> {
        Ok(())
    }
}
