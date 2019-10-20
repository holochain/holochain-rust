use crate::interface::{Interface, RpcHandler};
use conductor::broadcaster::Broadcaster;
use crossbeam_channel::Receiver;
use jsonrpc_ipc_server::{RequestContext, ServerBuilder};
use jsonrpc_pubsub::Session;
use std::{path::PathBuf, sync::Arc, thread};

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
        handler: RpcHandler,
        kill_switch: Receiver<()>,
    ) -> Result<(Broadcaster, thread::JoinHandle<()>), String> {
        let path_str = self.path.to_str().ok_or("Invalid socket path")?;
        let server = ServerBuilder::with_meta_extractor(handler, |context: &RequestContext| {
            Some(Arc::new(Session::new(context.sender.clone())))
        })
        .start(path_str)
        .map_err(|e| e.to_string())?;
        let broadcaster = Broadcaster::Noop;
        let handle = thread::Builder::new()
            .name(format!("unix_socket_interface/{:?}", path_str))
            .spawn(move || {
                let _ = server; // move `server` into this thread
                let _ = kill_switch.recv();
            })
            .expect("Could not spawn thread for domain socket interface");
        Ok((broadcaster, handle))
    }
}
