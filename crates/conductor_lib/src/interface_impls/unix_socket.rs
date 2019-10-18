use conductor::broadcaster::Broadcaster;
use crossbeam_channel::Receiver;
use interface::Interface;
use jsonrpc_core::IoHandler;
use jsonrpc_ipc_server::ServerBuilder;
use std::thread;

pub struct UnixSocketInterface {
    file: PathBuf,
}

impl UnixSocketInterface {
    pub fn new(file: PathBuf) -> Self {
        UnixSocketInterface { file }
    }
}

impl Interface for UnixSocketInterface {
    fn run(
        &self,
        handler: IoHandler,
        kill_switch: Receiver<()>,
    ) -> Result<(Broadcaster, thread::JoinHandle<()>), String> {
        let url = format!("0.0.0.0:{}", self.file);
        let server = ServerBuilder::new(handler)
            .start(self.file)
            .map_err(|e| e.to_string())?;
        let stream = UnixStream::connect(self.file.into())
            .map_err(|e| format!("Could not establish Unix domain socket! {:?}", e))?;
        let broadcaster = Broadcaster::UnixSocket(stream);
        let handle = thread::Builder::new()
            .name(format!("unix_socket_interface/{}", url))
            .spawn(move || {
                let _ = server; // move `server` into this thread
                let _ = kill_switch.recv();
            })
            .expect("Could not spawn thread for websocket interface");
        Ok((broadcaster, handle))
    }
}
