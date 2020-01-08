use crate::*;
use std::io::{Error, ErrorKind, Result};

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

    pub fn process(&mut self) -> Result<bool> {
        match self.process_inner() {
            Ok(did_work) => Ok(did_work),
            Err(e) => {
                let err: std::io::Error = e.kind().clone().into();
                self.send_wss_event
                    .f_send(WssEvent::Error(self.url.clone(), err.into()));
                Err(e)
            }
        }
    }

    /// allows us to capture any errors and forward the error back
    fn process_inner(&mut self) -> Result<bool> {
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
            self.send_wss_event
                .f_send(WssEvent::Error(self.url.clone(), e.into()));
        }
    }

    fn check_commands(&mut self) -> Result<bool> {
        let mut did_work = false;

        // process a batch of outgoing messages at a time
        for _ in 0..100 {
            match self.recv_wss_command.try_recv() {
                Ok(cmd) => {
                    did_work = true;
                    match cmd {
                        WssCommand::CloseConnection(_url) => {
                            // TODO - send closing websocket frame
                            return Err(Error::new(ErrorKind::Other, "closing"));
                        }
                        WssCommand::SendMessage(_url, frame) => {
                            self.send_message(frame);
                        }
                    }
                }
                Err(crossbeam_channel::TryRecvError::Disconnected) => {
                    //panic!("broken recv_wss_command channel");
                    // if the channel is disconnected, we got closed
                    return Err(Error::new(ErrorKind::Other, "closing"));
                }
                Err(crossbeam_channel::TryRecvError::Empty) => break,
            }
        }

        Ok(did_work)
    }

    fn check_incoming_messages(&mut self) -> Result<bool> {
        let mut did_work = false;

        if self.frame.is_none() {
            self.frame = Some(WsFrame::default());
        }

        match self.wss.read(self.frame.as_mut().unwrap()) {
            Ok(_) => {
                did_work = true;
                let frame = self.frame.take().unwrap();
                trace!("frame read from {} {:?}", self.url, frame);
                self.send_wss_event
                    .f_send(WssEvent::ReceivedData(self.url.clone(), frame));
            }
            Err(e) if e.would_block() => (),
            Err(e) => {
                error!("error in read for {}: {:?}", self.url, e);
                return Err(e);
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

    pub fn exec(&mut self) -> bool {
        let mut did_work = false;

        if self.check_new_connections() {
            did_work = true;
        }

        if self.check_commands() {
            did_work = true;
        }

        did_work
    }

    fn check_new_connections(&mut self) -> bool {
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
                    panic!("broken recv_new_connection channel");
                }
                Err(crossbeam_channel::TryRecvError::Empty) => break,
            }
        }

        did_work
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
                    panic!("broken recv_wss_command channel");
                }
                Err(crossbeam_channel::TryRecvError::Empty) => break,
            }
        }

        did_work
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
        if !connections.exec() {
            last_break = std::time::Instant::now();
            futures_timer::Delay::new(std::time::Duration::from_millis(5)).await;
        }

        if last_break.elapsed().as_millis() > 20 {
            last_break = std::time::Instant::now();
            // equivalent of thread::yield_now() ?
            futures::future::lazy(|_| {}).await;
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
                            send_connection_item.f_send(item);
                        }
                        Ok(false) => {
                            send_connection_item.f_send(item);
                        }
                        // the item should already have sent back the error
                        Err(_) => (),
                    }
                }
                Err(crossbeam_channel::TryRecvError::Disconnected) => {
                    panic!("broken recv_connection_item channel");
                }
                Err(crossbeam_channel::TryRecvError::Empty) => break,
            }

            if last_break.elapsed().as_millis() > 20 {
                last_break = std::time::Instant::now();
                // equivalent of thread::yield_now() ?
                futures::future::lazy(|_| {}).await;
            }
        }

        if !did_work {
            last_break = std::time::Instant::now();
            futures_timer::Delay::new(std::time::Duration::from_millis(5)).await;
        }
    }
}
