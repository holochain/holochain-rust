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
}

impl ConnectionJob {
    pub(crate) fn new(
        wss: TcpWss,
        msg_send: crossbeam_channel::Sender<(Url2, FrameResult)>,
    ) -> Self {
        Self {
            cont: true,
            wss,
            msg_send,
            frame: None,
        }
    }

    /// cancel this job - will be dropped next time it is polled.
    pub(crate) fn stop(&mut self) {
        self.cont = false;
    }

    /// send data out on this websocket connection
    pub(crate) fn send(&mut self, msg: WsFrame) -> Sim2hResult<()> {
        self.wss.write(msg)?;
        Ok(())
    }

    /// internal - report a received message or error
    fn report_msg(&self, msg: FrameResult) {
        self.msg_send.f_send((self.wss.remote_url(), msg));
    }

    fn run(&mut self) -> JobContinue {
        if !self.cont {
            return false;
        }
        if self.frame.is_none() {
            self.frame = Some(WsFrame::default());
        }
        match self.wss.read(self.frame.as_mut().unwrap()) {
            Ok(_) => {
                let frame = self.frame.take().unwrap();
                self.report_msg(Ok(frame));
            }
            Err(e) if e.would_block() => (),
            Err(e) => {
                error!("WEBSOCKET ERROR: {:?}", e);
                self.report_msg(Err(e.into()));
                return false;
            }
        }
        true
    }
}

impl Job for Arc<Mutex<ConnectionJob>> {
    fn run(&mut self) -> JobContinue {
        self.f_lock().run()
    }
}
