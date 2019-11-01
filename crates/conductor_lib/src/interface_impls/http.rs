use conductor::broadcaster::Broadcaster;
use crossbeam_channel::Receiver;
use interface::Interface;
use jsonrpc_core::IoHandler;
use jsonrpc_http_server::ServerBuilder;
use std::thread;

pub struct HttpInterface {
    port: u16,
}

impl HttpInterface {
    pub fn new(port: u16) -> Self {
        HttpInterface { port }
    }
}

impl Interface for HttpInterface {
    fn run(
        &self,
        handler: IoHandler,
        kill_switch: Receiver<()>,
    ) -> Result<(Broadcaster, thread::JoinHandle<()>), String> {
        let url = format!("0.0.0.0:{}", self.port);

        let server = ServerBuilder::new(handler)
            .start_http(&url.parse().expect("Invalid URL!"))
            .map_err(|e| e.to_string())?;
        let broadcaster = Broadcaster::Noop;
        let handle = thread::Builder::new()
            .name(format!("http_interface/{}", url))
            .spawn(move || {
                let _ = server; // move `server` into this thread
                let _ = kill_switch.recv();
            })
            .expect("Could not spawn thread for HTTP interface");
        Ok((broadcaster, handle))
    }
}
