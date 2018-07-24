use std::collections::HashMap;
use std::collections::hash_map::Entry;

use zmq;
use rmp_serde;
use serde;

use context;
use msg_types::*;
use errors::*;
use util::*;
use message::*;

pub type SendResult = Box<FnMut(Result<MsgSrvRespOk>) -> Result<()> + Send>;
pub type CallResult = Box<FnMut(Result<MsgSrvRespOk>) -> Result<()> + Send>;
pub type CallResponseResult = Box<FnMut(Result<MsgSrvRecvCallResp>) -> Result<()> + Send>;

pub struct IpcClient {
    socket: zmq::Socket,
    next_id: u64,
    send_callbacks: HashMap<Vec<u8>, SendResult>,
    call_callbacks: HashMap<Vec<u8>, CallResult>,
    call_resp_callbacks: HashMap<Vec<u8>, CallResponseResult>,
}

impl IpcClient {
    pub fn new () -> Result<Self> {
        Ok(Self {
            socket: context::socket(zmq::ROUTER)?,
            next_id: 0,
            send_callbacks: HashMap::new(),
            call_callbacks: HashMap::new(),
            call_resp_callbacks: HashMap::new(),
        })
    }

    pub fn close (mut self) -> Result<()> {
        drop(self.socket);
        self.send_callbacks.clear();
        self.call_callbacks.clear();
        self.call_resp_callbacks.clear();
        Ok(())
    }

    pub fn connect (&mut self, endpoint: &str) -> Result<()> {
        let connect_start = get_millis();
        self.socket.connect(endpoint)?;
        loop {
            if get_millis() - connect_start > 1000.0 {
                return Err(IpcError::Timeout.into());
            }

            self.ping()?;

            match self.process(10)? {
                Some(msg) => {
                    match msg {
                        Message::SrvPong(pong) => {
                            println!("got pong: toServerMs: {}, roundTripMs: {}",
                                (pong.1 - pong.0).round() as i64,
                                (get_millis() - pong.0).round() as i64);
                            break;
                        }
                        _ => {
                            panic!("cannot handle non-pongs during connect");
                        }
                    }
                }
                None => continue,
            }
        }
        Ok(())
    }

    pub fn ping (&mut self) -> Result<()> {
        let ping = get_millis();
        self.priv_send(MSG_CLI_PING, &ping)?;
        Ok(())
    }

    pub fn send (&mut self, to_address: &[u8], data: &[u8], cb: SendResult) -> Result<()> {
        let id = self.priv_get_id()?;
        self.send_callbacks.insert(id.clone(), cb);
        let snd = MsgCliSend(&id, to_address, data);
        self.priv_send(MSG_CLI_SEND, &snd)?;
        Ok(())
    }

    pub fn call (&mut self, to_address: &[u8], data: &[u8], cb: CallResult, resp_cb: CallResponseResult) -> Result<()> {
        let id = self.priv_get_id()?;
        self.call_callbacks.insert(id.clone(), cb);
        self.call_resp_callbacks.insert(id.clone(), resp_cb);
        let snd = MsgCliCall(&id, &id, to_address, data);
        self.priv_send(MSG_CLI_CALL, &snd)?;
        Ok(())
    }

    pub fn process (&mut self, millis: i64) -> Result<Option<Message>> {
        let res = self.socket.poll(zmq::POLLIN, millis)?;
        if res == 0 {
            return Ok(None);
        }

        // we have data, let's fetch it
        let res = self.socket.recv_multipart(0)?;
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
                let resp: MsgSrvRespOk = rmp_serde::from_slice(msg)?;
                match self.send_callbacks.entry(resp.0.clone()) {
                    Entry::Occupied(mut e) => {
                        e.get_mut()(Ok(resp.clone()))?;
                        e.remove();
                    }
                    _ => ()
                }
                match self.call_callbacks.entry(resp.0.clone()) {
                    Entry::Occupied(mut e) => {
                        e.get_mut()(Ok(resp.clone()))?;
                        e.remove();
                    }
                    _ => ()
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
                match self.call_resp_callbacks.entry(recv.0.clone()) {
                    Entry::Occupied(mut e) => {
                        e.get_mut()(Ok(recv.clone()))?;
                        e.remove();
                    }
                    _ => ()
                }
                return Ok(Some(Message::SrvRecvCallResp(recv)));
            }
            _ => panic!("unexpected message type: {}", t),
        }
    }

    // -- private -- //

    fn priv_get_id (&mut self) -> Result<Vec<u8>> {
        self.next_id += 1;
        return Ok(rmp_serde::to_vec(&(self.next_id - 1))?);
    }

    fn priv_send<T> (&mut self, t: u8, data: &T) -> Result<()>
    where T: serde::Serialize {
        let mut data = rmp_serde::to_vec(data)?;
        data.insert(0, t);
        self.socket.send_multipart(&[
            &[0x24, 0x24, 0x24, 0x24],
            &[],
            &data
        ], 0)?;
        Ok(())
    }
}
