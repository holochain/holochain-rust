use crate::*;

/// a job that manages the sim2h listening socket connection
/// every iteration will `accept()` a single pending connection
pub(crate) struct ListenJob {
    listen: TcpWssServer,
    wss_send: crossbeam_channel::Sender<TcpWss>,
}

impl ListenJob {
    pub(crate) fn new(listen: TcpWssServer, wss_send: crossbeam_channel::Sender<TcpWss>) -> Self {
        Self { listen, wss_send }
    }

    fn run(&mut self) -> JobContinue {
        match self.listen.accept() {
            Ok(wss) => {
                self.wss_send.f_send(wss);
            }
            Err(e) if e.would_block() => (),
            Err(e) => {
                error!("LISTEN ACCEPT FAIL: {:?}", e);
                //return false;
                // uhh... this is fatal for now
                panic!(e);
            }
        }
        true
    }
}

impl Job for Arc<Mutex<ListenJob>> {
    fn run(&mut self) -> JobContinue {
        self.f_lock().run()
    }
}
