use crate::*;

/// ConnectionJob periodically calls `read` on the underlying websocket stream
/// if there is data or an error, will forward a FrameResult
pub(crate) type FrameResult = Result<WsFrame, Sim2hError>;

/// manages a websocket stream/socket - will periodically poll for data
pub(crate) struct ConnectionJob {
    cont: bool,
    wss: TcpWss,
    msg_send: crossbeam_channel::Sender<(Url2, FrameResult)>,
    frame: Option<WsFrame>,
    outgoing_recv: crossbeam_channel::Receiver<WsFrame>,
}

impl ConnectionJob {
    pub(crate) fn new(
        wss: TcpWss,
        msg_send: crossbeam_channel::Sender<(Url2, FrameResult)>,
    ) -> (Self, crossbeam_channel::Sender<WsFrame>) {
        let (outgoing_send, outgoing_recv) = crossbeam_channel::unbounded();
        (
            Self {
                cont: true,
                wss,
                msg_send,
                frame: None,
                outgoing_recv,
            },
            outgoing_send,
        )
    }

    /// cancel this job - will be dropped next time it is polled.
    pub(crate) fn stop(&mut self) {
        self.cont = false;
    }

    /// internal - report a received message or error
    fn report_msg(&self, msg: FrameResult) {
        self.msg_send.f_send((self.wss.remote_url(), msg));
    }

    fn run(&mut self) -> JobResult {
        if !self.cont {
            return JobResult::done();
        }
        if self.frame.is_none() {
            self.frame = Some(WsFrame::default());
        }
        if let Ok(frame) = self.outgoing_recv.try_recv() {
            if let Err(e) = self.wss.write(frame) {
                error!("WEBSOCKET ERROR: {:?}", e);
                self.report_msg(Err(e.into()));
                return JobResult::done();
            }
        }
        match self.wss.read(self.frame.as_mut().unwrap()) {
            Ok(_) => {
                let frame = self.frame.take().unwrap();
                self.report_msg(Ok(frame));
                // we got data this time, check again right away
                return JobResult::default();
            }
            Err(e) if e.would_block() => (),
            Err(e) => {
                error!("WEBSOCKET ERROR: {:?}", e);
                self.report_msg(Err(e.into()));
                return JobResult::done();
            }
        }
        // no data this round, wait 5ms before checking again
        JobResult::default().wait_ms(5)
    }
}

impl Job for Arc<Mutex<ConnectionJob>> {
    fn run(&mut self) -> JobResult {
        self.f_lock().run()
    }
}
