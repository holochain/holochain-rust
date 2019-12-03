//! unix domain sockets on *nix + macOs systems
//! named pipes on windows

use crate::*;
use std::io::{Error, ErrorKind, Read, Result, Write};
use url2::prelude::*;

const SCHEME: &'static str = "ipc";

fn ipc_url_to_path(url: &Url2) -> Result<std::path::PathBuf> {
    if url.scheme() != SCHEME {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!("got: '{}', expected: '{}:///path/to/socket'", url, SCHEME),
        ));
    }
    let mut path = std::path::PathBuf::new();
    path.push(url.path());
    Ok(path)
}

#[derive(Debug)]
/// configuration options for ipc bind
pub struct IpcBindConfig {
}

impl Default for IpcBindConfig {
    fn default() -> Self {
        Self {
        }
    }
}

impl InStreamConfig for IpcBindConfig {}

#[derive(Debug)]
pub struct InStreamListenerIpc {
    #[cfg(not(windows))]
    pub listener: std::os::unix::net::UnixListener,
    pub binding: Url2,
}

impl InStreamListenerIpc {
    pub fn bind(url: &Url2, config: IpcBindConfig) -> Result<Self> {
        InStreamListenerIpc::raw_bind(url, config)
    }
}

impl InStreamListener<&mut [u8], &[u8]> for InStreamListenerIpc {
    type Stream = InStreamIpc;

    fn raw_bind<C: InStreamConfig>(url: &Url2, config: C) -> Result<Self> {
        let _config = IpcBindConfig::from_gen(config)?;
        let path = ipc_url_to_path(url)?;
        let listener = std::os::unix::net::UnixListener::bind(path)?;
        listener.set_nonblocking(true)?;
        Ok(InStreamListenerIpc {
            listener,
            binding: url.clone(),
        })
    }

    fn binding(&self) -> Url2 {
        self.binding.clone()
    }

    fn accept(&mut self) -> Result<<Self as InStreamListener<&mut [u8], &[u8]>>::Stream> {
        let (stream, _addr) = self.listener.accept()?;
        stream.set_nonblocking(true)?;
        Ok(InStreamIpc {
            stream,
        })
    }
}

#[derive(Debug)]
/// configuration options for ipc connect
pub struct IpcConnectConfig {
}

impl Default for IpcConnectConfig {
    fn default() -> Self {
        Self {
        }
    }
}

impl InStreamConfig for IpcConnectConfig {}

#[cfg(not(windows))]
#[derive(Shrinkwrap, Debug)]
#[shrinkwrap(mutable)]
pub struct InStreamIpc {
    #[shrinkwrap(main_field)]
    pub stream: std::os::unix::net::UnixStream,
}

impl InStreamIpc {
    pub fn connect(url: &Url2, config: IpcConnectConfig) -> Result<Self> {
        InStreamIpc::raw_connect(url, config)
    }
}

impl InStream<&mut [u8], &[u8]> for InStreamIpc {
    const URL_SCHEME: &'static str = SCHEME;

    #[cfg(not(windows))]
    fn raw_connect<C: InStreamConfig>(url: &Url2, config: C) -> Result<Self> {
        let _config = IpcConnectConfig::from_gen(config)?;
        let path = ipc_url_to_path(url)?;
        let stream = std::os::unix::net::UnixStream::connect(path)?;
        stream.set_nonblocking(true)?;
        Ok(Self {
            stream,
        })
    }

    #[cfg(not(windows))]
    fn read(&mut self, data: &mut [u8]) -> Result<usize> {
        self.stream.read(data)
    }

    #[cfg(not(windows))]
    fn write(&mut self, data: &[u8]) -> Result<usize> {
        let written = self.stream.write(data)?;
        if written != data.len() {
            // TODO - buffer?
            panic!("failed to write all");
        }
        Ok(written)
    }

    #[cfg(not(windows))]
    fn flush(&mut self) -> Result<()> {
        self.stream.flush()
    }
}

impl InStreamStd for InStreamIpc {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ipc_works() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_string_lossy().to_string();

        let (send_binding, recv_binding) = crossbeam_channel::unbounded();

        let server_thread = std::thread::spawn(move || {
            let mut listener = InStreamListenerIpc::bind(
                &url2!("ipc://{}/in-stream-ipc-test.socket", path),
                IpcBindConfig::default(),
            )
            .unwrap();
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

            let mut cli = InStreamIpc::connect(&binding, IpcConnectConfig::default())
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

        dir.close().unwrap();

        println!("done");
    }
}
