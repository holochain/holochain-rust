use crate::*;

/// ConnectionJob periodically calls `read` on the underlying websocket stream
/// if there is data or an error, will forward a FrameResult
pub(crate) type FrameResult = Result<WsFrame, Sim2hError>;

struct ConnectionMgr {
    recv_new_connection: crossbeam_channel::Receiver<TcpWss>,
    send_incoming_message: crossbeam_channel::Sender<(Url2, FrameResult)>,
    recv_outgoing_message: crossbeam_channel::Receiver<(Url2, WsFrame)>,
    // needs to NOT be an IM HashMap, as we cannot clone sockets : )
    connections: std::collections::HashMap<Url2, TcpWss>,
    frame: Option<WsFrame>,
}

impl ConnectionMgr {
    pub fn new(
        recv_new_connection: crossbeam_channel::Receiver<TcpWss>,
        send_incoming_message: crossbeam_channel::Sender<(Url2, FrameResult)>,
        recv_outgoing_message: crossbeam_channel::Receiver<(Url2, WsFrame)>,
    ) -> Self {
        Self {
            recv_new_connection,
            send_incoming_message,
            recv_outgoing_message,
            connections: std::collections::HashMap::new(),
            frame: None,
        }
    }

    pub fn exec(&mut self) -> bool {
        let mut did_work = false;

        if self.check_new_connections() {
            did_work = true;
        }

        if self.check_outgoing_messages() {
            did_work = true;
        }

        if self.check_incoming_messages() {
            did_work = true;
        }

        did_work
    }

    fn report_msg(&self, url: Url2, msg: FrameResult) {
        self.send_incoming_message.f_send((url, msg));
    }

    fn check_new_connections(&mut self) -> bool {
        let mut did_work = false;

        // we cannot check for outgoing messages until we have processed
        // ALL incoming connections (otherwise we might fail to send something)

        loop {
            match self.recv_new_connection.try_recv() {
                Ok(wss) => {
                    self.connections.insert(wss.remote_url(), wss);
                    did_work = true;
                }
                Err(crossbeam_channel::TryRecvError::Disconnected) => {
                    panic!("broken recv_new_connection channel");
                }
                Err(crossbeam_channel::TryRecvError::Empty) => break,
            }
        }

        did_work
    }

    fn check_outgoing_messages(&mut self) -> bool {
        let mut did_work = false;

        // process a batch of outgoing messages at a time
        for _ in 0..100 {
            match self.recv_outgoing_message.try_recv() {
                Ok((url, frame)) => {
                    if let Some(wss) = self.connections.get_mut(&url) {
                        if let Err(e) = wss.write(frame) {
                            error!("WEBSOCKET ERROR-outgoing: {:?}", e);
                            self.connections.remove(&url);
                            self.report_msg(url, Err(e.into()));
                        }
                    } else {
                        let err = format!("no route to send {} message {:?}", url, frame);
                        warn!("{}", err);
                        self.report_msg(url, Err(err.into()));
                    }
                    did_work = true;
                }
                Err(crossbeam_channel::TryRecvError::Disconnected) => {
                    panic!("broken recv_outgoing_message channel");
                }
                Err(crossbeam_channel::TryRecvError::Empty) => break,
            }
        }

        did_work
    }

    fn check_incoming_messages(&mut self) -> bool {
        let mut did_work = false;

        let mut reports = Vec::new();
        let mut removes = Vec::new();

        // do a single loop of checking for incoming messages
        // note - would be better to use mio-style epoll here
        for (url, wss) in self.connections.iter_mut() {
            if self.frame.is_none() {
                self.frame = Some(WsFrame::default());
            }

            match wss.read(self.frame.as_mut().unwrap()) {
                Ok(_) => {
                    did_work = true;
                    let frame = self.frame.take().unwrap();
                    trace!("frame read {} {:?}", url, frame);
                    reports.push((url.clone(), Ok(frame)));
                }
                Err(e) if e.would_block() => (),
                Err(e) => {
                    did_work = true;
                    error!("WEBSOCKET ERROR-read: {:?}", e);
                    reports.push((url.clone(), Err(e.into())));
                    removes.push(url.clone());
                }
            }
        }

        for (url, msg) in reports.drain(..) {
            self.report_msg(url, msg);
        }

        for url in removes.iter() {
            self.connections.remove(url);
        }

        did_work
    }
}

/// process all open connections (send / receive data)
/// timing strategy:
///   - while there are new connections, keep going for 10 ms, then yield
///   - if WouldBlock, sleep for 5 ms
#[allow(dead_code)]
async fn connection_job(
    recv_new_connection: crossbeam_channel::Receiver<TcpWss>,
    send_incoming_message: crossbeam_channel::Sender<(Url2, FrameResult)>,
    recv_outgoing_message: crossbeam_channel::Receiver<(Url2, WsFrame)>,
) {
    let mut last_break = std::time::Instant::now();
    let mut connections = ConnectionMgr::new(
        recv_new_connection,
        send_incoming_message,
        recv_outgoing_message,
    );
    loop {
        if !connections.exec() {
            last_break = std::time::Instant::now();
            futures_timer::Delay::new(std::time::Duration::from_millis(5)).await;
        }

        if last_break.elapsed().as_millis() > 10 {
            last_break = std::time::Instant::now();
            // equivalent of thread::yield_now() ?
            futures::future::lazy(|_| {}).await;
        }
    }
}

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
        match self.run_result() {
            Ok(job_result) => job_result,
            Err(e) => {
                self.report_msg(Err(e));
                // got connection error - stop this job
                JobResult::done()
            }
        }
    }

    fn run_result(&mut self) -> Result<JobResult, Sim2hError> {
        if !self.cont {
            return Ok(JobResult::done());
        }
        if self.frame.is_none() {
            self.frame = Some(WsFrame::default());
        }
        match self.outgoing_recv.try_recv() {
            Ok(frame) => {
                if let Err(e) = self.wss.write(frame) {
                    error!("WEBSOCKET ERROR-outgoing: {:?}", e);
                    return Err(e.into());
                }
            }
            Err(crossbeam_channel::TryRecvError::Disconnected) => {
                error!("parent channel disconnect");
                return Err("parent channel disconnect".into());
            }
            Err(crossbeam_channel::TryRecvError::Empty) => (),
        }
        match self.wss.read(self.frame.as_mut().unwrap()) {
            Ok(_) => {
                let frame = self.frame.take().unwrap();
                trace!("frame read {:?}", frame);
                self.report_msg(Ok(frame));
                // we got data this time, check again right away
                return Ok(JobResult::default());
            }
            Err(e) if e.would_block() => (),
            Err(e) => {
                error!("WEBSOCKET ERROR-read: {:?}", e,);
                return Err(e.into());
            }
        }
        // no data this round, wait 5ms before checking again
        Ok(JobResult::default().wait_ms(5))
    }
}

impl Job for Arc<Mutex<ConnectionJob>> {
    fn run(&mut self) -> JobResult {
        self.f_lock().run()
    }
}
