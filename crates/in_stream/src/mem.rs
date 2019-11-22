use crate::*;
use std::{
    collections::{hash_map::Entry, HashMap},
    io::{Error, ErrorKind, Read, Result, Write},
};
use url2::prelude::*;

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

/// memory connection specific bind config
pub struct MemBindConfig {}

impl Default for MemBindConfig {
    fn default() -> Self {
        Self {}
    }
}

impl InStreamListener for InStreamListenerMem {
    type Partial = InStreamPartialMem;
    type BindConfig = MemBindConfig;

    fn bind(url: &Url2, _config: Self::BindConfig) -> Result<Self> {
        get_mem_manager().bind(url)
    }

    fn binding(&self) -> Url2 {
        self.url.clone()
    }

    fn accept(&mut self) -> Result<<Self as InStreamListener>::Partial> {
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
        Ok(InStreamPartialMem(Some(self.accept_queue.remove(0))))
    }
}

/// a partly connected memory stream
#[derive(Debug)]
pub struct InStreamPartialMem(Option<InStreamMem>);

/// memory stream specific connect config
pub struct MemConnectConfig {}

impl Default for MemConnectConfig {
    fn default() -> Self {
        Self {}
    }
}

impl InStreamPartial for InStreamPartialMem {
    type Stream = InStreamMem;
    type ConnectConfig = MemConnectConfig;

    /// we want a url like mem://
    const URL_SCHEME: &'static str = "mem";

    fn with_stream(stream: Self::Stream) -> Result<Self> {
        Ok(Self(Some(stream)))
    }

    fn connect(url: &Url2, _config: Self::ConnectConfig) -> Result<Self> {
        Ok(Self(Some(get_mem_manager().connect(url)?)))
    }

    fn process(&mut self) -> Result<Self::Stream> {
        match self.0.take() {
            None => Err(Error::new(ErrorKind::NotFound, "raw stream is None")),
            Some(stream) => Ok(stream),
        }
    }
}

/// a singleton memory transport
/// could be used for unit testing or for in-process ipc
#[derive(Debug)]
pub struct InStreamMem {
    url: Url2,
    send: crossbeam_channel::Sender<Vec<u8>>,
    recv: crossbeam_channel::Receiver<Vec<u8>>,
    recv_buf: Vec<u8>,
}

impl InStream for InStreamMem {}

impl InStreamMem {
    /// private constructor, you probably want `connect`
    fn priv_new(
        url: Url2,
        send: crossbeam_channel::Sender<Vec<u8>>,
        recv: crossbeam_channel::Receiver<Vec<u8>>,
    ) -> Self {
        Self {
            url,
            send,
            recv,
            recv_buf: Vec::new(),
        }
    }
}

impl Read for InStreamMem {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut disconnected = false;
        loop {
            // first, drain everything from our channel
            match self.recv.try_recv() {
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
}

impl Write for InStreamMem {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        // if we're still connected, send data to our pair
        match self.send.send(buf.to_vec()) {
            Ok(_) => Ok(buf.len()),
            Err(_) => Err(ErrorKind::NotConnected.into()),
        }
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

// -- utility functions -- //

pub mod in_stream_mem {
    use super::*;

    /// create a unique url for binding an InStreamListenerMem instance
    pub fn random_url(prefix: &str) -> Url2 {
        Url2::parse(&format!(
            "mem://{}-{}",
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
        if "mem" != url.scheme() {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "mem bind: url scheme must be mem",
            ));
        }
        match url.port() {
            None | Some(0) | Some(4242) => (),
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "mem bind: url port must be None, 0, or 4242",
                ));
            }
        }
        if url.host_str().is_none() {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "mem bind: host_str must be set",
            ));
        }
        let new_url = Url2::parse(&format!("mem://{}:4242", url.host_str().unwrap(),));
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
        let url = if url.scheme() != "mem" || url.host_str().is_none() {
            Url2::parse(&format!("mem://{}", url,))
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
        let mut l =
            InStreamListenerMem::bind(&random_url("test"), MemBindConfig::default()).unwrap();
        println!("bound to: {}", l.binding());

        let mut c = InStreamPartialMem::connect(&l.binding(), MemConnectConfig::default()).unwrap();

        let mut srv = l.accept_blocking().unwrap().process_blocking().unwrap();

        let mut cli = c.process_blocking().unwrap();

        let mut buf = [0; 32];

        srv.write_all(b"hello from server").unwrap();
        cli.write_all(b"hello from client").unwrap();

        std::thread::sleep(std::time::Duration::from_millis(100));

        assert_eq!(17, srv.read(&mut buf).unwrap());
        assert_eq!("hello from client", &String::from_utf8_lossy(&buf[..17]));
        assert_eq!(17, cli.read(&mut buf).unwrap());
        assert_eq!("hello from server", &String::from_utf8_lossy(&buf[..17]));

        println!("done");
    }
}
