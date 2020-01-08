use crate::*;

const PENDING_CONNECTION_TIMEOUT_MS: usize = 30_000; // 30 seconds

/// wait for connections to complete handshaking && timeout slow / errant
/// timing strategy:
///   - while we did work, keep going for 10 ms, then yield
///   - if no work was done, sleep for 5 ms
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

        if !pending.exec() {
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

#[allow(clippy::large_enum_variant)]
enum PendingState {
    Pending(PendingItem),
    Ready(TcpWss),
    Error(Url2, std::io::Error),
}

struct PendingItem {
    start: std::time::Instant,
    wss: TcpWss,
}

impl PendingItem {
    pub fn new(wss: TcpWss) -> Self {
        Self {
            start: std::time::Instant::now(),
            wss,
        }
    }

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

    pub fn exec(&mut self) -> bool {
        let mut did_work = false;

        if self.check_new_connections() {
            did_work = true;
        }

        if self.check_pending_connections() {
            did_work = true;
        }

        did_work
    }

    fn check_new_connections(&mut self) -> bool {
        let mut did_work = false;

        // process a batch of incoming connections
        for _ in 0..100 {
            match self.recv_pending.try_recv() {
                Ok(wss) => {
                    did_work = true;
                    self.connections.push(PendingItem::new(wss));
                }
                Err(crossbeam_channel::TryRecvError::Disconnected) => {
                    panic!("broken recv_pending channel");
                }
                Err(crossbeam_channel::TryRecvError::Empty) => break,
            }
        }

        did_work
    }

    fn check_pending_connections(&mut self) -> bool {
        let mut did_work = false;

        let to_check = self.connections.drain(..).collect::<Vec<_>>();
        for item in to_check {
            match item.check() {
                PendingState::Pending(item) => {
                    self.connections.push(item);
                }
                PendingState::Ready(wss) => {
                    did_work = true;
                    self.send_ready.f_send(wss);
                }
                PendingState::Error(url, e) => {
                    did_work = true;
                    warn!("Pending Connection Handshake Failed {} {:?}", url, e);
                }
            }
        }

        did_work
    }
}
