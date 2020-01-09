use crate::*;

const PENDING_CONNECTION_TIMEOUT_MS: usize = 30_000; // 30 seconds

/// wait for connections to complete handshaking && timeout slow / errant
/// timing strategy:
///   - if there are any pending connections, we assume handshaking work
///     is happening - yield after ~ 20 ms
///   - if there are no pending connections, sleep for 5 ms
pub(crate) async fn pending_job(
    recv_pending: crossbeam_channel::Receiver<TcpWss>,
    send_ready: crossbeam_channel::Sender<TcpWss>,
) {
    let mut last_debug = std::time::Instant::now();
    let mut last_break = std::time::Instant::now();
    let mut pending = PendingMgr::new(recv_pending, send_ready);
    loop {
        if last_debug.elapsed().as_secs() >= 1 {
            last_debug = std::time::Instant::now();
            debug!("pending connection count: {}", pending.connections.len());
        }

        match pending.exec() {
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

#[allow(clippy::large_enum_variant)]
/// result of checking the state on a pending websocket connection
enum PendingState {
    Pending(PendingItem),
    Ready(TcpWss),
    Error(Url2, std::io::Error),
}

/// a websocket connection that may not have completed handshaking
struct PendingItem {
    // the time at which we received this socket
    start: std::time::Instant,
    // the actual socket
    wss: TcpWss,
}

impl PendingItem {
    pub fn new(wss: TcpWss) -> Self {
        Self {
            start: std::time::Instant::now(),
            wss,
        }
    }

    /// check to see if this websocket has completed handshaking
    pub fn check(mut self) -> PendingState {
        match self.wss.check_ready() {
            Ok(true) => return PendingState::Ready(self.wss),
            Err(e) => return PendingState::Error(self.wss.remote_url(), e),
            _ => (),
        }
        if self.start.elapsed().as_millis() as usize > PENDING_CONNECTION_TIMEOUT_MS {
            return PendingState::Error(self.wss.remote_url(), std::io::ErrorKind::TimedOut.into());
        }
        PendingState::Pending(self)
    }
}

/// internal helper struct for tracking pending websocket connections
struct PendingMgr {
    recv_pending: crossbeam_channel::Receiver<TcpWss>,
    send_ready: crossbeam_channel::Sender<TcpWss>,
    connections: Vec<PendingItem>,
}

impl PendingMgr {
    pub fn new(
        recv_pending: crossbeam_channel::Receiver<TcpWss>,
        send_ready: crossbeam_channel::Sender<TcpWss>,
    ) -> Self {
        Self {
            recv_pending,
            send_ready,
            connections: Vec::new(),
        }
    }

    pub fn exec(&mut self) -> Result<bool, ()> {
        let mut did_work = false;

        if self.check_new_connections()? {
            did_work = true;
        }

        if !self.connections.is_empty() {
            did_work = true;
            self.check_pending_connections()?;
        }

        Ok(did_work)
    }

    fn check_new_connections(&mut self) -> Result<bool, ()> {
        let mut did_work = false;

        // process a batch of incoming connections
        for _ in 0..100 {
            match self.recv_pending.try_recv() {
                Ok(wss) => {
                    did_work = true;
                    self.connections.push(PendingItem::new(wss));
                }
                Err(crossbeam_channel::TryRecvError::Disconnected) => {
                    error!("pending job recv_pending disconnected");
                    return Err(());
                }
                Err(crossbeam_channel::TryRecvError::Empty) => break,
            }
        }

        Ok(did_work)
    }

    fn check_pending_connections(&mut self) -> Result<(), ()> {
        let to_check = self.connections.drain(..).collect::<Vec<_>>();
        for item in to_check {
            match item.check() {
                PendingState::Pending(item) => {
                    self.connections.push(item);
                }
                PendingState::Ready(wss) => {
                    if !self.send_ready.i_send(wss) {
                        return Err(());
                    }
                }
                PendingState::Error(url, e) => {
                    warn!("Pending Connection Handshake Failed {} {:?}", url, e);
                }
            }
        }
        Ok(())
    }
}
