//! implements a net_connection::NetWorker for messaging with an ipc p2p node

use crate::{socket::IpcSocket, util::get_millis};
use holochain_net_connection::{
    net_connection::{NetHandler, NetReceive},
    protocol::{NamedBinaryData, PingData, PongData, Protocol},
    NetResult,
};
use snowflake::ProcessUniqueId;
use std::{thread, time};

// with two zmq "ROUTER" sockets, one side must have a well-known id
// for the holochain ipc protocol, the server is always 4 0x24 bytes
static SRV_ID: &'static [u8] = &[0x24, 0x24, 0x24, 0x24];

/// NetWorker for messaging with an ipc p2p node
pub struct IpcClient {
    handler: NetHandler,
    socket: Box<IpcSocket>,
    last_recv_millis: f64,
    last_send_millis: f64,
    id: ProcessUniqueId,
}

impl NetReceive for IpcClient {
    /// stop the worker
    fn stop(self: Box<Self>) -> NetResult<()> {
        self.socket.close()?;
        Ok(())
    }

    /// send message sent to us from holochain_net to the ipc server handling the network
    fn receive(&mut self, data: Protocol) -> NetResult<()> {
        match data {
            Protocol::NamedBinary(_) =>  println!(">>>> IpcClient send: {:?}", data),
            Protocol::Json(_) =>  println!(">>>> IpcClient send: {:?}", data),
           _ => (),
        };
        self.priv_send(&data)
    }

    /// perform upkeep (like ping/pong messages) on the underlying ipc socket
    fn tick(&mut self) -> NetResult<bool> {
        let mut did_something = false;
        if let Some(msg) = self.priv_proc_message()? {
            did_something = true;
            if let Protocol::Ping(ref p) = msg {
                // received Ping from network
                // => send back a Pong
                self.priv_send(&Protocol::Pong(PongData {
                    orig: p.sent,
                    recv: get_millis(),
                }))?;
            }
            match msg {
                Protocol::NamedBinary(_) => println!("<<<< IpcClient recv: {:?}", msg),
                Protocol::Json(_) => println!("<<<< IpcClient recv: {:?}", msg),
                _ => (),
            };
            (self.handler)(Ok(msg))?;
        }
        let now = get_millis();
        if now - self.last_recv_millis > 2000.0 {
            bail!(format!("[{}] ipc connection timeout", self.id))
        }
        if now - self.last_send_millis > 500.0 {
            self.priv_ping()?;
            did_something = true;
        }
        Ok(did_something)
    }

    fn endpoint(&self) -> Option<String> {
        self.socket.endpoint()
    }
}

impl IpcClient {
    /// establish a new ipc connection
    /// for now, the api simplicity is worth blocking the thread on connection
    pub fn new(
        handler: NetHandler,
        mut socket: Box<IpcSocket>,
        block_connect: bool,
    ) -> NetResult<Self> {
        if block_connect {
            let start = get_millis();
            let mut backoff = 1_u64;

            loop {
                // wait for any message from server to indicate connect success
                if socket.poll(0)? {
                    break;
                }

                if get_millis() - start > 3000.0 {
                    bail!("connection init timeout");
                }

                let data = Protocol::Ping(PingData { sent: get_millis() });
                let data: NamedBinaryData = data.into();
                socket.send(&[SRV_ID, &[], &b"ping".to_vec(), &data.data])?;

                backoff *= 2;
                if backoff > 500 {
                    backoff = 500;
                }

                thread::sleep(time::Duration::from_millis(backoff));
            }
        }
        Ok(Self {
            handler,
            socket,
            last_recv_millis: get_millis(),
            last_send_millis: 0.0,
            id: ProcessUniqueId::new(),
        })
    }

    // -- private -- //

    /// monitor the ipc socket / handle messages
    fn priv_proc_message(&mut self) -> NetResult<Option<Protocol>> {
        if !self.socket.poll(0)? {
            return Ok(None);
        }

        // we have data, let's fetch it
        let res = self.socket.recv()?;
        if res.len() != 4 {
            bail!("bad msg len: {}", res.len());
        }

        // we got a message, update our timeout counter
        self.last_recv_millis = get_millis();

        let msg = NamedBinaryData {
            name: res[2].to_vec(),
            data: res[3].to_vec(),
        };

        let msg: Protocol = msg.into();

        // TODO: use logger instead
        // println!("[{}] priv_proc_message() msg = {:?}", self.id, msg);
        Ok(Some(msg))
    }

    /// Send a heartbeat message to the ipc server.
    fn priv_ping(&mut self) -> NetResult<()> {
        self.priv_send(&Protocol::Ping(PingData { sent: get_millis() }))
    }

    /// send a raw message to the ipc server
    fn priv_send(&mut self, data: &Protocol) -> NetResult<()> {
        let data: NamedBinaryData = data.into();

        // TODO: use logger instead
        // println!("[{}] priv_send() data = {:?}", self.id, data.name);
        self.socket.send(&[SRV_ID, &[], &data.name, &data.data])?;

        // sent message, update our ping timer
        self.last_send_millis = get_millis();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::mpsc;

    use crate::socket::{make_test_channels, MockIpcSocket};

    #[test]
    fn it_ipc_message_flow() {
        let (sender, receiver) = mpsc::channel::<Protocol>();

        let (test_struct, stx, srx) = make_test_channels().unwrap();
        let s = MockIpcSocket::new_test(test_struct).unwrap();

        let pong = Protocol::Pong(PongData {
            orig: get_millis() - 4.0,
            recv: get_millis() - 2.0,
        });
        let data: NamedBinaryData = (&pong).into();

        stx.send(vec![vec![], vec![], b"pong".to_vec(), data.data])
            .unwrap();

        let mut cli = Box::new(
            IpcClient::new(
                Box::new(move |r| {
                    sender.send(r?)?;
                    Ok(())
                }),
                s,
                true,
            )
            .unwrap(),
        );

        cli.tick().unwrap();

        let res = receiver.recv().unwrap();

        assert_eq!(pong, res);

        stx.send(vec![]).unwrap();;

        cli.tick().expect_err("expected bad arg count");

        let ping = Protocol::Ping(PingData { sent: get_millis() });
        let data: NamedBinaryData = (&ping).into();

        stx.send(vec![vec![], vec![], b"ping".to_vec(), data.data])
            .unwrap();

        cli.tick().unwrap();

        let res = receiver.recv().unwrap();

        assert_eq!(ping, res);

        cli.receive(Protocol::P2pReady).unwrap();

        let mut out: Vec<String> = Vec::new();

        loop {
            thread::sleep(time::Duration::from_millis(2));

            let res = srx.recv().unwrap();
            if res[2].as_slice() == b"ping" {
                continue;
            } else {
                out.push(String::from_utf8_lossy(&res[2]).to_string());
                if out.len() >= 2 {
                    break;
                }
            }
        }

        out.sort_unstable();

        assert_eq!("p2pReady", &out[0]);
        assert_eq!("pong", &out[1]);

        cli.tick().unwrap();
        cli.stop().unwrap();
    }
}
