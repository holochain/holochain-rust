use crate::*;
use std::{
    collections::{hash_map::Entry, HashMap},
    io::{Error, ErrorKind, Result},
};
use url2::prelude::*;

const SCHEME: &'static str = "mem";
const PORT: u16 = 4242;

/// bind to a virtual in-process ipc "interface"
#[derive(Debug)]
pub struct InStreamListenerMem {
    url: Url2,
    recv: crossbeam_channel::Receiver<InStreamMem>,
    accept_queue: Vec<InStreamMem>,
}

impl InStreamListenerMem {
    /// private constructor, you probably want `bind`
    fn priv_new(url: Url2, recv: crossbeam_channel::Receiver<InStreamMem>) -> Self {
        Self {
            url,
            recv,
            accept_queue: Vec::new(),
        }
    }
}

impl Drop for InStreamListenerMem {
    fn drop(&mut self) {
        get_mem_manager().unbind(&self.url);
    }
}

/*
/// memory connection specific bind config
pub struct MemBindConfig {}

impl Default for MemBindConfig {
    fn default() -> Self {
        Self {}
    }
}
*/

impl InStreamListener<&mut [u8], &[u8]> for InStreamListenerMem {
    type Stream = InStreamMem;

    fn raw_bind<C: InStreamConfig>(url: &Url2, _config: C) -> Result<Self> {
        get_mem_manager().bind(url)
    }

    fn binding(&self) -> Url2 {
        self.url.clone()
    }

    fn accept(&mut self) -> Result<<Self as InStreamListener<&mut [u8], &[u8]>>::Stream> {
        loop {
            // first, drain all pending connections from our recv channel
            match self.recv.try_recv() {
                Ok(stream) => {
                    self.accept_queue.push(stream);
                }
                Err(crossbeam_channel::TryRecvError::Empty) => break,
                Err(crossbeam_channel::TryRecvError::Disconnected) => {
                    // wait until our user has accepted all pending connections
                    // before letting them know the channel is broken
                    if self.accept_queue.is_empty() {
                        return Err(ErrorKind::BrokenPipe.into());
                    }
                }
            }
        }
        if self.accept_queue.is_empty() {
            // acceptor is non-blocking we have nothing to return
            return Err(Error::with_would_block());
        }
        // pull the next item off the queue

        Ok(self.accept_queue.remove(0))
    }
}

impl InStreamListenerStd for InStreamListenerMem {
    type StreamStd = InStreamMem;

    fn accept_std(&mut self) -> Result<<Self as InStreamListenerStd>::StreamStd> {
        self.accept()
    }
}

/*
/// memory stream specific connect config
pub struct MemConnectConfig {}

impl Default for MemConnectConfig {
    fn default() -> Self {
        Self {}
    }
}
*/

/// a singleton memory transport
/// could be used for unit testing or for in-process ipc
#[derive(Debug)]
pub struct InStreamMem {
    url: Url2,
    send: Option<crossbeam_channel::Sender<Vec<u8>>>,
    recv: Option<crossbeam_channel::Receiver<Vec<u8>>>,
    recv_buf: Vec<u8>,
}

impl InStream<&mut [u8], &[u8]> for InStreamMem {
    /// we want a url like mem://
    const URL_SCHEME: &'static str = SCHEME;

    fn raw_connect<C: InStreamConfig>(url: &Url2, _config: C) -> Result<Self> {
        Ok(get_mem_manager().connect(url)?)
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut disconnected = false;
        if let Some(recv) = &mut self.recv {
            for _ in 0..100 {
                // first, drain up to 100 non-blocking items from our channel
                match recv.try_recv() {
                    Ok(mut data) => {
                        self.recv_buf.append(&mut data);
                    }
                    Err(crossbeam_channel::TryRecvError::Empty) => break,
                    Err(crossbeam_channel::TryRecvError::Disconnected) => {
                        // if our channel is broken, we will consider it EOF
                        disconnected = true;
                        break;
                    }
                }
            }
        }
        if self.recv_buf.len() == 0 {
            if disconnected {
                // nothing in our buffer, let the user know about the EOF
                return Ok(0);
            } else {
                // nothing in our buffer, but our channel is still active
                // let them know that we have no data without blocking
                return Err(Error::with_would_block());
            }
        }

        // drain as much as we have and / or the user can take
        let v: Vec<u8> = self
            .recv_buf
            .drain(0..std::cmp::min(buf.len(), self.recv_buf.len()))
            .collect();
        buf[0..v.len()].copy_from_slice(&v);
        Ok(v.len())
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        match &mut self.send {
            None => Err(ErrorKind::NotConnected.into()),
            Some(send) => {
                // if we're still connected, send data to our pair
                match send.send(buf.to_vec()) {
                    Ok(_) => Ok(buf.len()),
                    Err(_) => Err(ErrorKind::NotConnected.into()),
                }
            }
        }
    }

    fn flush(&mut self) -> Result<()> {
        if self.send.is_none() {
            return Err(ErrorKind::NotConnected.into());
        }
        Ok(())
    }
}

impl InStreamStd for InStreamMem {}

impl InStreamMem {
    /// private constructor, you probably want `connect`
    fn priv_new(
        url: Url2,
        send: crossbeam_channel::Sender<Vec<u8>>,
        recv: crossbeam_channel::Receiver<Vec<u8>>,
    ) -> Self {
        Self {
            url,
            send: Some(send),
            recv: Some(recv),
            recv_buf: Vec::new(),
        }
    }

    pub fn shutdown(&mut self, how: std::net::Shutdown) -> Result<()> {
        match how {
            std::net::Shutdown::Read => {
                self.recv.take();
            }
            std::net::Shutdown::Write => {
                self.send.take();
            }
            std::net::Shutdown::Both => {
                self.recv.take();
                self.send.take();
            }
        }
        Ok(())
    }
}

// -- utility functions -- //

pub mod in_stream_mem {
    use super::*;

    /// create a unique url for binding an InStreamListenerMem instance
    pub fn random_url(prefix: &str) -> Url2 {
        Url2::parse(&format!(
            "{}://{}-{}",
            SCHEME,
            prefix,
            nanoid::simple().replace("_", "-").replace("~", "+"),
        ))
    }
}

use in_stream_mem::random_url;

/// private stream pair constructor, these streams can message each other
fn create_mem_stream_pair(url_a: Url2, url_b: Url2) -> (InStreamMem, InStreamMem) {
    let (send1, recv1) = crossbeam_channel::unbounded();
    let (send2, recv2) = crossbeam_channel::unbounded();
    (
        InStreamMem::priv_new(url_a, send1, recv2),
        InStreamMem::priv_new(url_b, send2, recv1),
    )
}
// -- singleton memory manager -- //

/// private singleton for managing virtual memory listening interfaces
struct MemManager {
    listeners: HashMap<Url2, crossbeam_channel::Sender<InStreamMem>>,
}

impl MemManager {
    /// create a new singleton
    fn new() -> Self {
        Self {
            listeners: HashMap::new(),
        }
    }

    /// manage binding a new MemListener interface
    fn bind(&mut self, url: &Url2) -> Result<InStreamListenerMem> {
        if SCHEME != url.scheme() {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!("mem bind: url scheme must be '{}'", SCHEME),
            ));
        }
        match url.port() {
            None | Some(0) | Some(PORT) => (),
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    format!("mem bind: url port must be None, 0, or {}", PORT),
                ));
            }
        }
        if url.host_str().is_none() {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "mem bind: host_str must be set",
            ));
        }
        let new_url = Url2::parse(&format!(
            "{}://{}:{}",
            SCHEME,
            url.host_str().unwrap(),
            PORT
        ));
        match self.listeners.entry(new_url.clone()) {
            Entry::Occupied(_) => Err(ErrorKind::AddrInUse.into()),
            Entry::Vacant(e) => {
                // the url is not in use, let's create a new listener
                let (send, recv) = crossbeam_channel::unbounded();
                e.insert(send);
                Ok(InStreamListenerMem::priv_new(new_url, recv))
            }
        }
    }

    /// unbind a previously bound MemListener interface (happens on Drop)
    fn unbind(&mut self, url: &Url2) {
        self.listeners.remove(url);
    }

    /// connect to an existing MemListener interface
    fn connect(&mut self, url: &Url2) -> Result<InStreamMem> {
        let url = if url.scheme() != SCHEME || url.host_str().is_none() {
            Url2::parse(&format!("{}://{}", SCHEME, url))
        } else {
            url.clone()
        };

        let mut disconnected = false;
        if let Entry::Occupied(mut e) = self.listeners.entry(url.clone()) {
            // there is a listener bound to this url
            // create a new stream pair
            // send one to the listener's accept queue
            // return the other one
            let (one, two) = create_mem_stream_pair(random_url("assigned"), url.clone());
            // if the send fails, we must have a broken listener connection
            // we'll clean that up after
            match e.get_mut().send(one) {
                Ok(_) => return Ok(two),
                Err(_) => disconnected = true,
            }
        }
        if disconnected {
            self.listeners.remove(&url);
        }
        Err(ErrorKind::ConnectionRefused.into())
    }
}

// this is the actual singleton global reference
lazy_static! {
    static ref MEM_MANAGER: parking_lot::Mutex<MemManager> =
        { parking_lot::Mutex::new(MemManager::new()) };
}

fn get_mem_manager() -> parking_lot::MutexGuard<'static, MemManager> {
    MEM_MANAGER
        .try_lock_for(std::time::Duration::from_secs(10))
        .expect("failed to acquire mem manager lock")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mem_works() {
        use std::io::{Read, Write};

        let (send_binding, recv_binding) = crossbeam_channel::unbounded();

        let server_thread = std::thread::spawn(move || {
            let mut listener =
                InStreamListenerMem::raw_bind(&random_url("test"), ()).unwrap();
            println!("bound to: {}", listener.binding());
            send_binding.send(listener.binding()).unwrap();

            let mut srv = loop {
                match listener.accept() {
                    Ok(srv) => break srv,
                    Err(e) if e.would_block() => {
                        std::thread::sleep(std::time::Duration::from_millis(1));
                    }
                    Err(e) => panic!("{:?}", e),
                }
            }
            .into_std_stream();

            srv.write(b"hello from server").unwrap();
            srv.flush().unwrap();
            srv.shutdown(std::net::Shutdown::Write).unwrap();

            let mut res = String::new();
            loop {
                match srv.read_to_string(&mut res) {
                    Ok(_) => break,
                    Err(e) if e.would_block() => {
                        std::thread::sleep(std::time::Duration::from_millis(1));
                    }
                    Err(e) => panic!("{:?}", e),
                }
            }
            assert_eq!("hello from client", &res);
        });

        let client_thread = std::thread::spawn(move || {
            let binding = recv_binding.recv().unwrap();
            println!("connect to: {}", binding);

            let mut cli = InStreamMem::raw_connect(&binding, ())
                .unwrap()
                .into_std_stream();

            cli.write(b"hello from client").unwrap();
            cli.flush().unwrap();
            cli.shutdown(std::net::Shutdown::Write).unwrap();

            let mut res = String::new();
            loop {
                match cli.read_to_string(&mut res) {
                    Ok(_) => break,
                    Err(e) if e.would_block() => {
                        std::thread::sleep(std::time::Duration::from_millis(1));
                    }
                    Err(e) => panic!("{:?}", e),
                }
            }
            assert_eq!("hello from server", &res);
        });

        server_thread.join().unwrap();
        client_thread.join().unwrap();

        println!("done");
    }
}
