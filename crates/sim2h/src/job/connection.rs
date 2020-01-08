use crate::*;

/// commands control the connection manager job
pub(crate) enum WssCommand {
    /// close a connection (if open)
    CloseConnection(Url2),
    /// send an outgoing message (websocket frame) on an open connection
    SendMessage(Url2, WsFrame),
}

/// events emitted by the connection manager job
pub(crate) enum WssEvent {
    /// connection manager job recevied a new (already handshaken) connection
    IncomingConnection(Url2),
    /// connection manager received a websocket frame from an open connection
    ReceivedData(Url2, WsFrame),
    /// connection manager got an error sending or receiving on an open connection
    Error(Url2, Sim2hError),
}

/// internal connection manager helper struct
struct ConnectionMgr {
    // new (already handshaken) connections will be sent through this channel
    recv_new_connection: crossbeam_channel::Receiver<TcpWss>,
    // we will send events out on this channel
    send_wss_event: crossbeam_channel::Sender<WssEvent>,
    // we will receive commands on this channel
    recv_wss_command: crossbeam_channel::Receiver<WssCommand>,
    // needs to NOT be an IM HashMap, as we cannot clone sockets : )
    connections: std::collections::HashMap<Url2, TcpWss>,
    // storage for default receive frame
    frame: Option<WsFrame>,
}

impl ConnectionMgr {
    pub fn new(
        recv_new_connection: crossbeam_channel::Receiver<TcpWss>,
        send_wss_event: crossbeam_channel::Sender<WssEvent>,
        recv_wss_command: crossbeam_channel::Receiver<WssCommand>,
    ) -> Self {
        Self {
            recv_new_connection,
            send_wss_event,
            recv_wss_command,
            connections: std::collections::HashMap::new(),
            frame: None,
        }
    }

    pub fn exec(&mut self) -> bool {
        let mut did_work = false;

        if self.check_new_connections() {
            did_work = true;
        }

        if self.check_commands() {
            did_work = true;
        }

        if self.check_incoming_messages() {
            did_work = true;
        }

        did_work
    }

    fn check_new_connections(&mut self) -> bool {
        let mut did_work = false;

        // we cannot check for outgoing messages until we have processed
        // ALL incoming connections (otherwise we might fail to send something)

        loop {
            match self.recv_new_connection.try_recv() {
                Ok(wss) => {
                    did_work = true;
                    let url = wss.remote_url();
                    self.connections.insert(url.clone(), wss);
                    self.send_wss_event
                        .f_send(WssEvent::IncomingConnection(url));
                }
                Err(crossbeam_channel::TryRecvError::Disconnected) => {
                    panic!("broken recv_new_connection channel");
                }
                Err(crossbeam_channel::TryRecvError::Empty) => break,
            }
        }

        did_work
    }

    fn close_connection(&mut self, url: Url2) {
        // TODO - send close Wss frame
        self.connections.remove(&url);
    }

    fn send_message(&mut self, url: Url2, frame: WsFrame) {
        if let Some(wss) = self.connections.get_mut(&url) {
            if let Err(e) = wss.write(frame) {
                error!("WEBSOCKET ERROR-outgoing: {:?}", e);
                self.connections.remove(&url);
                self.send_wss_event.f_send(WssEvent::Error(url, e.into()));
            }
        } else {
            let err = format!("no route to send {} message {:?}", url, frame);
            warn!("{}", err);
            self.send_wss_event.f_send(WssEvent::Error(url, err.into()));
        }
    }

    fn check_commands(&mut self) -> bool {
        let mut did_work = false;

        // process a batch of outgoing messages at a time
        for _ in 0..100 {
            match self.recv_wss_command.try_recv() {
                Ok(cmd) => {
                    did_work = true;
                    match cmd {
                        WssCommand::CloseConnection(url) => {
                            self.close_connection(url);
                        }
                        WssCommand::SendMessage(url, frame) => {
                            self.send_message(url, frame);
                        }
                    }
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
                    reports.push(WssEvent::ReceivedData(url.clone(), frame));
                }
                Err(e) if e.would_block() => (),
                Err(e) => {
                    did_work = true;
                    error!("WEBSOCKET ERROR-read: {:?}", e);
                    reports.push(WssEvent::Error(url.clone(), e.into()));
                    removes.push(url.clone());
                }
            }
        }

        for msg in reports.drain(..) {
            self.send_wss_event.f_send(msg);
        }

        for url in removes.iter() {
            self.connections.remove(url);
        }

        did_work
    }
}

/// process all open connections (send / receive data)
/// timing strategy:
///   - while we did work, keep going for 10 ms, then yield
///   - if no work was done, sleep for 5 ms
pub(crate) async fn connection_job(
    recv_new_connection: crossbeam_channel::Receiver<TcpWss>,
    send_wss_event: crossbeam_channel::Sender<WssEvent>,
    recv_wss_command: crossbeam_channel::Receiver<WssCommand>,
) {
    let mut last_break = std::time::Instant::now();
    let mut connections = ConnectionMgr::new(recv_new_connection, send_wss_event, recv_wss_command);
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
