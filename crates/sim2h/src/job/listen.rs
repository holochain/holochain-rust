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

    fn run(&mut self) -> JobResult {
        match self.listen.accept() {
            Ok(wss) => {
                self.wss_send.f_send(wss);
                // we got data this time, check again right away
                return JobResult::default();
            }
            Err(e) if e.would_block() => (),
            Err(e) => {
                error!("LISTEN ACCEPT FAIL: {:?}", e);
                //return false;
                // uhh... this is fatal for now
                panic!(e);
            }
        }
        // no data this round, wait 5ms before checking again
        JobResult::default().wait_ms(5)
    }
}

impl Job for Arc<Mutex<ListenJob>> {
    fn run(&mut self) -> JobResult {
        self.f_lock().run()
    }
}
