//! This module represents a holochain application's inter-process-communication connection to an external p2p process.

use std::collections::{hash_map::Entry, HashMap};

use rmp_serde;
use serde;

use errors::*;
use message::*;
use msg_types::*;
use socket::{IpcSocket, ZmqIpcSocket};
use util::*;

/// A closure callback type def for getting acknowledgment when performing a `call`.
pub type CallResult = Box<FnMut(Result<Vec<u8>>) -> Result<()> + Send>;

/// IPC communication client structure. Allows connection to an external process that manages p2p communications.
///
/// This struct takes an abstract socket type mainly to facilitate unit testing. You will mainly instantiate the exported ZmqIpcClient type definition.
pub struct IpcClient<S: IpcSocket> {
    socket: Box<S>,
    next_id: u64,
    call_callbacks: HashMap<Vec<u8>, (f64, CallResult)>,
}

impl<S: IpcSocket> IpcClient<S> {
    /// Perform any underlying socket library cleanup. Call this before your application exits.
    pub fn destroy_context() -> Result<()> {
        S::destroy_context()?;
        Ok(())
    }

    /// Get a new IpcClient instance.
    pub fn new() -> Result<Self> {
        Ok(Self {
            socket: S::new()?,
            next_id: 0,
            call_callbacks: HashMap::new(),
        })
    }

    /// Close this specific IpcClient connection.
    pub fn close(mut self) -> Result<()> {
        self.socket.close()?;
        self.call_callbacks.clear();
        Ok(())
    }

    /// Connect this IpcClient to a p2p ipc server.
    pub fn connect(&mut self, endpoint: &str) -> Result<()> {
        let connect_start = get_millis();

        let mut wait_backoff: i64 = 1;

        self.socket.connect(endpoint)?;
        loop {
            if get_millis() - connect_start > 1000.0 {
                return Err(IpcError::Timeout.into());
            }

            println!("sending ping");
            self.ping()?;

            match self.process(wait_backoff)? {
                Some(msg) => match msg {
                    Message::Pong(pong) => {
                        println!(
                            "got pong: toServerMs: {}, roundTripMs: {}",
                            (pong.1 - pong.0).round() as i64,
                            (get_millis() - pong.0).round() as i64
                        );
                        break;
                    }
                    _ => {
                        panic!("cannot handle non-pongs during connect");
                    }
                },
                None => {
                    wait_backoff *= 2;
                    continue;
                }
            }
        }
        Ok(())
    }

    /// Send a heartbeat message to the ipc server.
    pub fn ping(&mut self) -> Result<()> {
        let snd = MsgPingSend(get_millis());
        self.priv_send(MSG_PING, &snd)?;
        Ok(())
    }

    /// invoke an RPC-style `call` on the ipc server
    pub fn call(&mut self, data: &[u8], cb: Option<CallResult>) -> Result<()> {
        let id = self.priv_get_id()?;
        if let Some(cb) = cb {
            self.call_callbacks.insert(id.clone(), (get_millis(), cb));
        }
        let snd = MsgCallSend(&id, data);
        self.priv_send(MSG_CALL, &snd)?;
        Ok(())
    }

    /// respond to a remote `call`
    pub fn respond(&mut self, message_id: &[u8], data: Result<&[u8]>) -> Result<()> {
        match data {
            Err(e) => {
                let e = format!("{}", e);
                let snd = MsgCallFailSend(message_id, e.as_bytes());
                self.priv_send(MSG_CALL_FAIL, &snd)?;
            }
            Ok(d) => {
                let snd = MsgCallOkSend(message_id, d);
                self.priv_send(MSG_CALL_OK, &snd)?;
            }
        }
        Ok(())
    }

    /// Allow IPC client to do any needed processing.
    /// This should be called regularly to make sure any maintenance tasks are executed properly, and to avoid incoming data backing up in memory.
    ///
    /// If there are no incoming messages waiting in the queue, `millis` indicates how long we should block waiting for one. It is perfectly valid to pass in `0` for `millis`.
    pub fn process(&mut self, millis: i64) -> Result<Option<Message>> {
        if !self.socket.poll(millis)? {
            return Ok(None);
        }

        // we have data, let's fetch it
        let res = self.socket.recv()?;
        if res.len() != 3 {
            bail_generic!("bad msg len: {}", res.len());
        }

        let (t, msg) = res[2].split_first().ok_or(IpcError::MissingDataError)?;
        match *t {
            MSG_PONG => {
                let pong: MsgPongRecv = rmp_serde::from_slice(msg)?;
                return Ok(Some(Message::Pong(pong)));
            }
            MSG_CALL => {
                let call: MsgCallRecv = rmp_serde::from_slice(msg)?;
                println!("got call: {:?}", call);
                return Ok(Some(Message::Call(call)));
            }
            MSG_CALL_OK => {
                let resp: MsgCallOkRecv = rmp_serde::from_slice(msg)?;
                if let Entry::Occupied(mut e) = self.call_callbacks.entry(resp.0.clone()) {
                    e.get_mut().1(Ok(resp.1.clone()))?;
                    e.remove();
                }
                return Ok(Some(Message::CallOk(resp)));
            }
            MSG_CALL_FAIL => {
                let resp: MsgCallFailRecv = rmp_serde::from_slice(msg)?;
                let id = resp.0;
                let resp = String::from_utf8_lossy(&resp.1).to_string();
                let resp = IpcError::GenericError { error: resp };
                if let Entry::Occupied(mut e) = self.call_callbacks.entry(id.clone()) {
                    e.get_mut().1(Err(resp.clone().into()))?;
                    e.remove();
                }
                return Err(resp.into());
            }
            _ => panic!("unexpected message type: 0x{:x}", t),
        }
    }

    // -- private -- //

    /// message ids in this module are just sequential u64-s
    fn priv_get_id(&mut self) -> Result<Vec<u8>> {
        self.next_id += 1;
        return Ok(rmp_serde::to_vec(&(self.next_id - 1))?);
    }

    /// send a raw message to the ipc server
    fn priv_send<T>(&mut self, t: u8, data: &T) -> Result<()>
    where
        T: serde::Serialize,
    {
        let mut data = rmp_serde::to_vec(data)?;
        data.insert(0, t);
        // with two zmq "ROUTER" sockets, one side must have a well-known id
        // for the holochain ipc protocol, the server is always 4 0x24 bytes
        self.socket.send(&[&[0x24, 0x24, 0x24, 0x24], &[], &data])?;
        Ok(())
    }
}

/// The ZeroMQ implementation of IpcClient.
pub type ZmqIpcClient = IpcClient<ZmqIpcSocket>;

#[cfg(test)]
mod tests {
    use super::*;
    use socket::MockIpcSocket;

    #[derive(Serialize, Debug, Clone, PartialEq)]
    pub struct MsgPongSend(pub f64, pub f64);

    impl IpcClient<MockIpcSocket> {
        fn priv_test_inject(&mut self, data: Vec<Vec<u8>>) {
            self.socket.inject_response(data);
        }

        fn priv_test_sent_count(&mut self) -> usize {
            self.socket.sent_count()
        }

        fn priv_test_next_sent(&mut self) -> Vec<Vec<u8>> {
            self.socket.next_sent()
        }
    }

    #[test]
    fn it_can_construct_and_destroy_sockets() {
        let ipc: IpcClient<MockIpcSocket> = IpcClient::new().unwrap();
        ipc.close().unwrap();
    }

    #[test]
    fn it_can_connect() {
        let mut ipc: IpcClient<MockIpcSocket> = IpcClient::new().unwrap();

        let pong = MsgPongSend(get_millis() - 4.0, get_millis() - 2.0);
        let mut data = rmp_serde::to_vec(&pong).unwrap();
        data.insert(0, MSG_PONG);

        ipc.priv_test_inject(vec![vec![], vec![], data]);

        ipc.connect("test-garbage").unwrap();
        ipc.close().unwrap();
    }

    #[test]
    fn it_can_receive_calls() {
        let mut ipc: IpcClient<MockIpcSocket> = IpcClient::new().unwrap();
        let data = MsgCallSend(b"", &[0x42]);
        let mut data = rmp_serde::to_vec(&data).unwrap();
        data.insert(0, MSG_CALL);
        ipc.priv_test_inject(vec![vec![], vec![], data]);
        let result = ipc.process(0);
        let result = result.unwrap().unwrap();
        let result = match result {
            Message::Call(s) => s,
            _ => panic!("bad message type"),
        };

        assert_eq!(vec![0x42], result.1);

        ipc.close().unwrap();
    }

    #[test]
    fn it_can_receive_call_oks() {
        let mut ipc: IpcClient<MockIpcSocket> = IpcClient::new().unwrap();
        let data = MsgCallOkSend(b"", &[0x42]);
        let mut data = rmp_serde::to_vec(&data).unwrap();
        data.insert(0, MSG_CALL_OK);
        ipc.priv_test_inject(vec![vec![], vec![], data]);
        let result = ipc.process(0);
        let result = result.unwrap().unwrap();
        let result = match result {
            Message::CallOk(s) => s,
            _ => panic!("bad message type"),
        };

        assert_eq!(vec![0x42], result.1);

        ipc.close().unwrap();
    }

    #[test]
    fn it_can_receive_call_fails() {
        let mut ipc: IpcClient<MockIpcSocket> = IpcClient::new().unwrap();
        let data = MsgCallFailSend(b"", &[0x42]);
        let mut data = rmp_serde::to_vec(&data).unwrap();
        data.insert(0, MSG_CALL_FAIL);
        ipc.priv_test_inject(vec![vec![], vec![], data]);
        let result = ipc.process(0);
        let result = result.expect_err("should have been an Err result");
        let result = format!("{}", result);
        assert_eq!("IpcError: B".to_string(), result);

        ipc.close().unwrap();
    }

    #[test]
    fn it_can_publish_ok_responses() {
        let mut ipc: IpcClient<MockIpcSocket> = IpcClient::new().unwrap();
        ipc.respond(&[0x42], Ok(&[0x99])).unwrap();
        assert_eq!(1, ipc.priv_test_sent_count());
        let mut sent = ipc.priv_test_next_sent();
        assert_eq!(3, sent.len());
        let sent = sent.remove(2);
        let sent: MsgCallOkRecv = rmp_serde::from_slice(&sent[1..]).unwrap();
        assert_eq!(vec![0x42], sent.0);
        assert_eq!(vec![0x99], sent.1);
        ipc.close().unwrap();
    }

    #[test]
    fn it_can_publish_fail_responses() {
        let mut ipc: IpcClient<MockIpcSocket> = IpcClient::new().unwrap();
        ipc.respond(
            &[0x42],
            Err(IpcError::GenericError {
                error: "test".to_string(),
            }.into()),
        ).unwrap();
        assert_eq!(1, ipc.priv_test_sent_count());
        let mut sent = ipc.priv_test_next_sent();
        assert_eq!(3, sent.len());
        let sent = sent.remove(2);
        let sent: MsgCallFailRecv = rmp_serde::from_slice(&sent[1..]).unwrap();
        assert_eq!(vec![0x42], sent.0);
        assert_eq!(b"IpcError: test".to_vec(), sent.1);
        ipc.close().unwrap();
    }
}
