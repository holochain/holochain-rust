use crate::*;
use std::io::{Error, ErrorKind};

#[derive(Debug)]
/// commands control the connection manager job
pub(crate) enum WssCommand {
    /// close a connection (if open)
    CloseConnection(Url2),
    /// send an outgoing message (websocket frame) on an open connection
    SendMessage(Url2, WsFrame),
}

#[derive(Debug)]
/// events emitted by the connection manager job
pub(crate) enum WssEvent {
    /// connection manager job recevied a new (already handshaken) connection
    IncomingConnection(Url2),
    /// connection manager received a websocket frame from an open connection
    ReceivedData(Url2, WsFrame),
    /// connection manager got an error sending or receiving on an open connection
    Error(Url2, Sim2hError),
}

struct ConnectionItem {
    send_wss_event: crossbeam_channel::Sender<WssEvent>,
    recv_wss_command: crossbeam_channel::Receiver<WssCommand>,
    url: Url2,
    wss: TcpWss,
    frame: Option<WsFrame>,
}

impl ConnectionItem {
    pub fn new(
        send_wss_event: crossbeam_channel::Sender<WssEvent>,
        recv_wss_command: crossbeam_channel::Receiver<WssCommand>,
        wss: TcpWss,
    ) -> Self {
        let url = wss.remote_url();
        info!("new connection {}", url);
        send_wss_event.f_send(WssEvent::IncomingConnection(url.clone()));
        Self {
            send_wss_event,
            recv_wss_command,
            url,
            wss,
            frame: None,
        }
    }

    fn report_err(&mut self, e: std::io::Error) -> Result<(), ()> {
        self.send_wss_event
            .i_send(WssEvent::Error(self.url.clone(), e.into()));
        Err(())
    }

    fn report_close(&mut self) -> Result<(), ()> {
        self.report_err(Error::new(ErrorKind::Other, "closing"))
    }

    /// allows us to capture any errors and forward the error back
    fn process(&mut self) -> Result<bool, ()> {
        let mut did_work = false;

        if self.check_commands()? {
            did_work = true;
        }

        if self.check_incoming_messages()? {
            did_work = true;
        }

        Ok(did_work)
    }

    fn send_message(&mut self, frame: WsFrame) {
        if let Err(e) = self.wss.write(frame) {
            error!("error in write to {}: {:?}", self.url, e);
            if !self
                .send_wss_event
                .i_send(WssEvent::Error(self.url.clone(), e.into()))
            {
                error!("write channel error");
            }
        }
    }

    fn check_commands(&mut self) -> Result<bool, ()> {
        let mut did_work = false;

        // process a batch of outgoing messages at a time
        for _ in 0..100 {
            match self.recv_wss_command.try_recv() {
                Ok(cmd) => {
                    did_work = true;
                    match cmd {
                        WssCommand::CloseConnection(_url) => {
                            // TODO - send closing websocket frame
                            self.report_close()?;
                        }
                        WssCommand::SendMessage(_url, frame) => {
                            self.send_message(frame);
                        }
                    }
                }
                Err(crossbeam_channel::TryRecvError::Disconnected) => {
                    //panic!("broken recv_wss_command channel");
                    // if the channel is disconnected, we got closed
                    self.report_close()?;
                }
                Err(crossbeam_channel::TryRecvError::Empty) => break,
            }
        }

        Ok(did_work)
    }

    fn check_incoming_messages(&mut self) -> Result<bool, ()> {
        let mut did_work = false;

        if self.frame.is_none() {
            self.frame = Some(WsFrame::default());
        }

        match self.wss.read(self.frame.as_mut().unwrap()) {
            Ok(_) => {
                did_work = true;
                let frame = self.frame.take().unwrap();
                trace!("frame read from {} {:?}", self.url, frame);
                if !self
                    .send_wss_event
                    .i_send(WssEvent::ReceivedData(self.url.clone(), frame))
                {
                    self.report_close()?;
                }
            }
            Err(e) if e.would_block() => (),
            Err(e) => {
                error!("error in read for {}: {:?}", self.url, e);
                self.report_err(e)?;
            }
        }

        Ok(did_work)
    }
}

/// internal connection manager helper struct
struct ConnectionMgr {
    // new (already handshaken) connections will be sent through this channel
    recv_new_connection: crossbeam_channel::Receiver<TcpWss>,
    // we will send events out on this channel
    send_wss_event: crossbeam_channel::Sender<WssEvent>,
    // we will receive commands on this channel
    recv_wss_command: crossbeam_channel::Receiver<WssCommand>,
    // sender to put a connection item job back into the queue
    send_connection_item: crossbeam_channel::Sender<ConnectionItem>,
    // refs to individual socket channels
    socket_channels: HashMap<Url2, crossbeam_channel::Sender<WssCommand>>,
}

impl ConnectionMgr {
    pub fn new(
        recv_new_connection: crossbeam_channel::Receiver<TcpWss>,
        send_wss_event: crossbeam_channel::Sender<WssEvent>,
        recv_wss_command: crossbeam_channel::Receiver<WssCommand>,
    ) -> Self {
        let (send_connection_item, recv_connection_item) = crossbeam_channel::unbounded();

        for _ in 0..num_cpus::get() {
            // spawn cpu count jobs to process individual socket connections
            sim2h_spawn_ok(connection_job_inner(
                recv_connection_item.clone(),
                send_connection_item.clone(),
            ));
        }

        Self {
            recv_new_connection,
            send_wss_event,
            recv_wss_command,
            send_connection_item,
            socket_channels: HashMap::new(),
        }
    }

    pub fn exec(&mut self) -> Result<bool, ()> {
        let mut did_work = false;

        if self.check_new_connections()? {
            did_work = true;
        }

        if self.check_commands()? {
            did_work = true;
        }

        Ok(did_work)
    }

    fn check_new_connections(&mut self) -> Result<bool, ()> {
        let mut did_work = false;

        for _ in 0..100 {
            match self.recv_new_connection.try_recv() {
                Ok(wss) => {
                    did_work = true;
                    let (send_wss_command, recv_wss_command) = crossbeam_channel::unbounded();
                    let url = wss.remote_url();
                    self.socket_channels.insert(url, send_wss_command);
                    self.send_connection_item.f_send(ConnectionItem::new(
                        self.send_wss_event.clone(),
                        recv_wss_command,
                        wss,
                    ));
                }
                Err(crossbeam_channel::TryRecvError::Disconnected) => {
                    return Err(());
                }
                Err(crossbeam_channel::TryRecvError::Empty) => break,
            }
        }

        Ok(did_work)
    }

    fn check_commands(&mut self) -> Result<bool, ()> {
        let mut did_work = false;

        // process a batch of outgoing messages at a time
        for _ in 0..100 {
            match self.recv_wss_command.try_recv() {
                Ok(cmd) => {
                    did_work = true;
                    match cmd {
                        WssCommand::CloseConnection(url) => {
                            // TODO forward so we send close frame / data?
                            self.socket_channels.remove(&url);
                        }
                        WssCommand::SendMessage(url, frame) => {
                            if let Some(snd) = self.socket_channels.get(&url) {
                                if let Err(_e) =
                                    snd.send(WssCommand::SendMessage(url.clone(), frame))
                                {
                                    // the channel got closed --
                                    // the ChannelItem should already have
                                    // sent a close message
                                    self.socket_channels.remove(&url);
                                }
                            } else {
                                warn!("failed to send to {}, no route - {:?}", url, frame);
                            }
                        }
                    }
                }
                Err(crossbeam_channel::TryRecvError::Disconnected) => {
                    return Err(());
                }
                Err(crossbeam_channel::TryRecvError::Empty) => break,
            }
        }

        Ok(did_work)
    }
}

/// process all open connections (send / receive data)
/// timing strategy:
///   - while we did work, keep going for 20 ms, then yield
///   - if no work was done, sleep for 5 ms
pub(crate) async fn connection_job(
    recv_new_connection: crossbeam_channel::Receiver<TcpWss>,
    send_wss_event: crossbeam_channel::Sender<WssEvent>,
    recv_wss_command: crossbeam_channel::Receiver<WssCommand>,
) {
    let mut last_break = std::time::Instant::now();
    let mut connections = ConnectionMgr::new(recv_new_connection, send_wss_event, recv_wss_command);
    loop {
        match connections.exec() {
            // did no work, sleep for 5 ms
            Ok(false) => {
                last_break = std::time::Instant::now();
                futures_timer::Delay::new(std::time::Duration::from_millis(5)).await;
            }
            // got error, exit the job
            Err(_) => return,
            _ => (),
        }

        if last_break.elapsed().as_millis() > 20 {
            last_break = std::time::Instant::now();
            TaskYield::new().await;
        }
    }
}

/// a job for processing individual socket connections (send / recv)
/// timing strategy:
///  - process a batch of sockets, yield if we take > 20 ms
///  - if there are no sockets / or we did no work, sleep for 5 ms
async fn connection_job_inner(
    recv_connection_item: crossbeam_channel::Receiver<ConnectionItem>,
    send_connection_item: crossbeam_channel::Sender<ConnectionItem>,
) {
    let mut last_break = std::time::Instant::now();
    loop {
        let mut did_work = false;

        for _ in 0..100 {
            match recv_connection_item.try_recv() {
                Ok(mut item) => {
                    match item.process() {
                        Ok(true) => {
                            did_work = true;
                            if !send_connection_item.i_send(item) {
                                return;
                            }
                        }
                        Ok(false) => {
                            if !send_connection_item.i_send(item) {
                                return;
                            }
                        }
                        // the item should already have sent back the error
                        Err(_) => (),
                    }
                }
                Err(crossbeam_channel::TryRecvError::Disconnected) => {
                    return;
                }
                Err(crossbeam_channel::TryRecvError::Empty) => break,
            }

            if last_break.elapsed().as_millis() > 20 {
                last_break = std::time::Instant::now();
                TaskYield::new().await;
            }
        }

        if !did_work {
            last_break = std::time::Instant::now();
            futures_timer::Delay::new(std::time::Duration::from_millis(5)).await;
        }
    }
}
