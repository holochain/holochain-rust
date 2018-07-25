/*!
This module represents a holochain application's inter-process-communication connection to an external p2p process.
*/

use std::collections::{hash_map::Entry, HashMap};

use rmp_serde;
use serde;

use errors::*;
use message::*;
use msg_types::*;
use socket::{IpcSocket, ZmqIpcSocket};
use util::*;

/// A closure callback type def for getting acknowledgment when performing a `send`.
pub type SendResult = Box<FnMut(Result<MsgSrvRespOk>) -> Result<()> + Send>;

/// A closure callback type def for getting acknowledgment when performing a `call`.
pub type CallResult = Box<FnMut(Result<MsgSrvRespOk>) -> Result<()> + Send>;

/// A closure callback type def for getting a response from a remote server when performing a `call`.
pub type CallResponseResult = Box<FnMut(Result<MsgSrvRecvCallResp>) -> Result<()> + Send>;

/**
IPC communication client structure. Allows connection to an external process that manages p2p communications.

This struct takes an abstract socket type mainly to facilitate unit testing. You will mainly instantiate the exported ZmqIpcClient type definition.
*/
pub struct IpcClient<S: IpcSocket> {
    socket: Box<S>,
    next_id: u64,
    send_callbacks: HashMap<Vec<u8>, SendResult>,
    call_callbacks: HashMap<Vec<u8>, CallResult>,
    call_resp_callbacks: HashMap<Vec<u8>, CallResponseResult>,
}

impl<S: IpcSocket> IpcClient<S> {
    /**
    Perform any underlying socket library cleanup. Call this before your application exits.
    */
    pub fn destroy_context() -> Result<()> {
        S::destroy_context()?;
        Ok(())
    }

    /**
    Get a new IpcClient instance.
    */
    pub fn new() -> Result<Self> {
        Ok(Self {
            socket: S::new()?,
            next_id: 0,
            send_callbacks: HashMap::new(),
            call_callbacks: HashMap::new(),
            call_resp_callbacks: HashMap::new(),
        })
    }

    /**
    Close this specific IpcClient connection.
    */
    pub fn close(mut self) -> Result<()> {
        self.socket.close()?;
        self.send_callbacks.clear();
        self.call_callbacks.clear();
        self.call_resp_callbacks.clear();
        Ok(())
    }

    /**
    Connect this IpcClient to a p2p ipc server.
    */
    pub fn connect(&mut self, endpoint: &str) -> Result<()> {
        let connect_start = get_millis();
        self.socket.connect(endpoint)?;
        loop {
            if get_millis() - connect_start > 1000.0 {
                return Err(IpcError::Timeout.into());
            }

            self.ping()?;

            match self.process(10)? {
                Some(msg) => match msg {
                    Message::SrvPong(pong) => {
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
                None => continue,
            }
        }
        Ok(())
    }

    /**
    Send a heartbeat message to the ipc server.
    */
    pub fn ping(&mut self) -> Result<()> {
        let ping = get_millis();
        self.priv_send(MSG_CLI_PING, &ping)?;
        Ok(())
    }

    /**
    Transmit a fire-and-forget `send` message to another node on the p2p network.
    */
    pub fn send(&mut self, to_address: &[u8], data: &[u8], cb: Option<SendResult>) -> Result<()> {
        let id = self.priv_get_id()?;
        if let Some(cb) = cb {
            self.send_callbacks.insert(id.clone(), cb);
        }
        let snd = MsgCliSend(&id, to_address, data);
        self.priv_send(MSG_CLI_SEND, &snd)?;
        Ok(())
    }

    /**
    Transmit an RPC-style `call` message to another node on the p2p network.
    */
    pub fn call(
        &mut self,
        to_address: &[u8],
        data: &[u8],
        cb: Option<CallResult>,
        resp_cb: Option<CallResponseResult>,
    ) -> Result<()> {
        let id = self.priv_get_id()?;
        if let Some(cb) = cb {
            self.call_callbacks.insert(id.clone(), cb);
        }
        if let Some(resp_cb) = resp_cb {
            self.call_resp_callbacks.insert(id.clone(), resp_cb);
        }
        let snd = MsgCliCall(&id, &id, to_address, data);
        self.priv_send(MSG_CLI_CALL, &snd)?;
        Ok(())
    }

    /**
    Allow IPC client to do any needed processing.
    This should be called regularly to make sure any maintenance tasks are executed properly, and to avoid incoming data backing up in memory.

    If there are no incoming messages waiting in the queue, `millis` indicates how long we should block waiting for one. It is perfectly valid to pass in `0` for `millis`.
    */
    pub fn process(&mut self, millis: i64) -> Result<Option<Message>> {
        if !self.socket.poll(millis)? {
            return Ok(None);
        }

        // we have data, let's fetch it
        let res = self.socket.recv()?;
        if res.len() != 3 {
            gerr!("bad msg len: {}", res.len());
        }

        let (t, msg) = res[2].split_first().ok_or(IpcError::NoneError)?;
        match *t {
            MSG_SRV_PONG => {
                let pong: MsgSrvPong = rmp_serde::from_slice(msg)?;
                return Ok(Some(Message::SrvPong(pong)));
            }
            MSG_SRV_RESP_OK => {
                println!("parsing: {:?}", msg);
                let resp: MsgSrvRespOk = rmp_serde::from_slice(msg)?;
                if let Entry::Occupied(mut e) = self.send_callbacks.entry(resp.0.clone()) {
                    e.get_mut()(Ok(resp.clone()))?;
                    e.remove();
                }
                if let Entry::Occupied(mut e) = self.call_callbacks.entry(resp.0.clone()) {
                    e.get_mut()(Ok(resp.clone()))?;
                    e.remove();
                }
                return Ok(Some(Message::SrvRespOk(resp)));
            }
            MSG_SRV_RECV_SEND => {
                let recv: MsgSrvRecvSend = rmp_serde::from_slice(msg)?;
                return Ok(Some(Message::SrvRecvSend(recv)));
            }
            MSG_SRV_RECV_CALL => {
                let recv: MsgSrvRecvCall = rmp_serde::from_slice(msg)?;
                return Ok(Some(Message::SrvRecvCall(recv)));
            }
            MSG_SRV_RECV_CALL_RESP => {
                let recv: MsgSrvRecvCallResp = rmp_serde::from_slice(msg)?;
                if let Entry::Occupied(mut e) = self.call_resp_callbacks.entry(recv.0.clone()) {
                    e.get_mut()(Ok(recv.clone()))?;
                    e.remove();
                }
                return Ok(Some(Message::SrvRecvCallResp(recv)));
            }
            _ => panic!("unexpected message type: {}", t),
        }
    }

    // -- private -- //

    fn priv_get_id(&mut self) -> Result<Vec<u8>> {
        self.next_id += 1;
        return Ok(rmp_serde::to_vec(&(self.next_id - 1))?);
    }

    fn priv_send<T>(&mut self, t: u8, data: &T) -> Result<()>
    where
        T: serde::Serialize,
    {
        let mut data = rmp_serde::to_vec(data)?;
        data.insert(0, t);
        self.socket.send(&[&[0x24, 0x24, 0x24, 0x24], &[], &data])?;
        Ok(())
    }
}

/// The ZeroMQ implementation of IpcClient.
pub type ZmqIpcClient = IpcClient<ZmqIpcSocket>;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_bytes;
    use socket::MockIpcSocket;

    impl IpcClient<MockIpcSocket> {
        fn priv_test_inject(&mut self, data: Vec<Vec<u8>>) {
            self.socket.inject_response(data);
        }
    }

    #[test]
    fn it_can_construct_and_destroy_sockets() {
        let ipc: IpcClient<MockIpcSocket> = IpcClient::new().unwrap();
        ipc.close().unwrap();
    }

    #[derive(Serialize, Debug, Clone, PartialEq)]
    pub struct TestSrvRecvSend<'a>(
        #[serde(with = "serde_bytes")] pub &'a [u8],
        #[serde(with = "serde_bytes")] pub &'a [u8],
    );

    #[test]
    fn it_can_receive_sends() {
        let mut ipc: IpcClient<MockIpcSocket> = IpcClient::new().unwrap();
        let data = TestSrvRecvSend(b"", &[0x42]);
        let mut data = rmp_serde::to_vec(&data).unwrap();
        data.insert(0, MSG_SRV_RECV_SEND);
        ipc.priv_test_inject(vec![vec![], vec![], data]);
        let result = ipc.process(0);
        let result = result.unwrap().unwrap();
        let result = match result {
            Message::SrvRecvSend(s) => s,
            _ => panic!("bad message type"),
        };

        assert_eq!(0x42, result.1[0]);

        ipc.close().unwrap();
    }
}
