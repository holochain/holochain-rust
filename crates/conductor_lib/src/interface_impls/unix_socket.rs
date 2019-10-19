use conductor::broadcaster::Broadcaster;
use crossbeam_channel::Receiver;
use interface::Interface;
use jsonrpc_core::IoHandler;
use jsonrpc_ipc_server::ServerBuilder;
use std::{os::unix::net::UnixStream, path::PathBuf, thread};

pub struct UnixSocketInterface {
    path: PathBuf,
}

impl UnixSocketInterface {
    pub fn new(path: PathBuf) -> Self {
        UnixSocketInterface { path }
    }
}

impl Interface for UnixSocketInterface {
    fn run(
        &self,
        handler: IoHandler,
        kill_switch: Receiver<()>,
    ) -> Result<(Broadcaster, thread::JoinHandle<()>), String> {
        let path_str = self.path.to_str().ok_or("Invalid socket path")?;
        let server = ServerBuilder::new(handler)
            .start(path_str)
            .map_err(|e| e.to_string())?;
        let stream = UnixStream::connect(path_str)
            .map_err(|e| format!("Could not establish Unix domain socket! {:?}", e))?;
        let broadcaster = Broadcaster::UnixSocket(stream);
        let handle = thread::Builder::new()
            .name(format!("unix_socket_interface/{:?}", self.path))
            .spawn(move || {
                let _ = server; // move `server` into this thread
                let _ = kill_switch.recv();
            })
            .expect("Could not spawn thread for websocket interface");
        Ok((broadcaster, handle))
    }
}
