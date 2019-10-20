use conductor::broadcaster::Broadcaster;
use crossbeam_channel::Receiver;
use interface::{Interface, RpcHandler};
use jsonrpc_pubsub::Session;
use jsonrpc_ws_server::{RequestContext, ServerBuilder};
use std::{sync::Arc, thread};

pub struct WebsocketInterface {
    port: u16,
}

impl WebsocketInterface {
    pub fn new(port: u16) -> Self {
        WebsocketInterface { port }
    }
}

impl Interface for WebsocketInterface {
    fn run(
        &self,
        handler: RpcHandler,
        kill_switch: Receiver<()>,
    ) -> Result<(Broadcaster, thread::JoinHandle<()>), String> {
        let url = format!("0.0.0.0:{}", self.port);
        let server = ServerBuilder::with_meta_extractor(handler.io, |context: &RequestContext| {
            Some(Arc::new(Session::new(context.sender().clone())))
        })
        .start(&url.parse().expect("Invalid URL!"))
        .map_err(|e| e.to_string())?;
        let broadcaster = Broadcaster::Noop;
        let handle = thread::Builder::new()
            .name(format!("websocket_interface/{}", url))
            .spawn(move || {
                let _ = server; // move `server` into this thread
                let _ = kill_switch.recv();
            })
            .expect("Could not spawn thread for websocket interface");
        Ok((broadcaster, handle))
    }
}
