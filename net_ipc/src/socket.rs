//! This module is a thin wrapper around a ZMQ socket
//! It allows us to easily mock it out for unit tests
//! as well as manage context with lazy_static!

use crate::{context, errors::*};
use std::sync::mpsc;
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
    fn poll(&mut self, timeout_ms: i64) -> Result<bool>;

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

    fn poll(&mut self, timeout_ms: i64) -> Result<bool> {
        Ok(self.socket.poll(zmq::POLLIN, timeout_ms)? != 0)
    }

    fn recv(&mut self) -> Result<Vec<Vec<u8>>> {
        Ok(self.socket.recv_multipart(0)?)
    }

    fn send(&mut self, data: &[&[u8]]) -> Result<()> {
        self.socket.send_multipart(data, 0)?;
        Ok(())
    }
}

/// helper for working with mock sockets
pub struct TestStruct {
    control_recv: mpsc::Receiver<Vec<Vec<u8>>>,
    control_send: mpsc::SyncSender<Vec<Vec<u8>>>,
}

/// This is a concrete implementation of the IpcSocket trait for use in testing
pub struct MockIpcSocket {
    resp_queue: Vec<Vec<Vec<u8>>>,
    channel: TestStruct,
}

/// helper to create mock socket channels
pub fn make_test_channels() -> Result<(
    TestStruct,
    mpsc::SyncSender<Vec<Vec<u8>>>,
    mpsc::Receiver<Vec<Vec<u8>>>,
)> {
    let (tx_in, rx_in) = mpsc::sync_channel(10);
    let (tx_out, rx_out) = mpsc::sync_channel(10);
    Ok((
        TestStruct {
            control_recv: rx_in,
            control_send: tx_out,
        },
        tx_in,
        rx_out,
    ))
}

impl MockIpcSocket {
    pub fn new_test(channel: TestStruct) -> Result<Box<Self>> {
        let mut out = MockIpcSocket::new()?;
        out.channel = channel;
        Ok(out)
    }
}

impl IpcSocket for MockIpcSocket {
    fn new() -> Result<Box<Self>> {
        let (tx, rx) = mpsc::sync_channel(10);
        Ok(Box::new(Self {
            resp_queue: Vec::new(),
            channel: TestStruct {
                control_recv: rx,
                control_send: tx,
            },
        }))
    }

    fn close(self: Box<Self>) -> Result<()> {
        Ok(())
    }

    fn connect(&mut self, _endpoint: &str) -> Result<()> {
        Ok(())
    }

    fn poll(&mut self, _millis: i64) -> Result<bool> {
        self.channel
            .control_recv
            .try_recv()
            .and_then(|r| {
                self.resp_queue.push(r);
                Ok(())
            })
            .unwrap_or(());
        Ok(!self.resp_queue.is_empty())
    }

    fn recv(&mut self) -> Result<Vec<Vec<u8>>> {
        self.poll(0)?;
        if !self.resp_queue.is_empty() {
            return Ok(self.resp_queue.remove(0));
        }
        Ok(self.channel.control_recv.recv()?)
    }

    fn send(&mut self, data: &[&[u8]]) -> Result<()> {
        let mut tmp: Vec<Vec<u8>> = Vec::new();
        for item in data {
            tmp.push(item.to_vec());
        }
        self.channel.control_send.send(tmp)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_zmq_cycle() {
        let mut c = ZmqIpcSocket::new().unwrap();
        c.connect("tcp://127.0.0.1:0").unwrap();
        c.poll(0).unwrap();
        c.send(&[&[]]).unwrap();
        c.close().unwrap();
    }

    #[test]
    fn it_mock_cycle() {
        let (test_struct, tx, rx) = make_test_channels().unwrap();
        let mut c = MockIpcSocket::new_test(test_struct).unwrap();
        c.connect("").unwrap();

        assert_eq!(false, c.poll(0).unwrap());

        c.send(&[&[1]]).unwrap();

        let tmp = rx.recv();
        assert_eq!(1, tmp.unwrap()[0][0]);

        tx.send(vec![vec![2]]).unwrap();

        assert_eq!(true, c.poll(0).unwrap());
        assert_eq!(2, c.recv().unwrap()[0][0]);

        c.close().unwrap();
    }
}
