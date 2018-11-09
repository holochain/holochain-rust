//! This module is a thin wrapper around a ZMQ socket
//! It allows us to easily mock it out for unit tests
//! as well as manage context with lazy_static!

use context;
use errors::*;
use zmq;

/// trait that allows zmq socket abstraction
pub trait IpcSocket {
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

/// This is a concrete implementation of the IpcSocket trait for use in testing
pub struct MockIpcSocket {
    resp_queue: Vec<Vec<Vec<u8>>>,
    sent_queue: Vec<Vec<Vec<u8>>>,
}

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

impl IpcSocket for MockIpcSocket {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_zmq_cycle() {
        let mut c = ZmqIpcSocket::new().unwrap();
        let uri = format!(
            "ipc://{}",
            mktemp::Temp::new_file()
                .unwrap()
                .to_path_buf()
                .to_string_lossy()
        );
        c.connect(&uri).unwrap();
        c.poll(0).unwrap();
        c.send(&[&[]]).unwrap();
        c.close().unwrap();
    }

    #[test]
    fn it_mock_cycle() {
        let mut c = MockIpcSocket::new().unwrap();
        c.connect("").unwrap();

        assert_eq!(false, c.poll(0).unwrap());
        assert_eq!(0, c.sent_count());

        c.send(&[&[1]]).unwrap();

        assert_eq!(1, c.sent_count());
        assert_eq!(1, c.next_sent()[0][0]);

        c.inject_response(vec![vec![2]]);

        assert_eq!(true, c.poll(0).unwrap());
        assert_eq!(2, c.recv().unwrap()[0][0]);

        c.close().unwrap();
    }
}
