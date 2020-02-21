use crate::*;
use std::sync::{Arc, Weak};

#[derive(Clone)]
pub struct ConnectionCount(Arc<tokio::sync::RwLock<usize>>);

impl ConnectionCount {
    pub fn new() -> Self {
        Self(Arc::new(tokio::sync::RwLock::new(0)))
    }

    pub async fn get(&self) -> usize {
        *self.0.read().await
    }
}

/// incoming messages from websockets
#[derive(Debug)]
pub enum ConMgrEvent {
    Disconnect(Lib3hUri, Option<Sim2hError>),
    ReceiveData(Lib3hUri, WsFrame),
}

/// messages for controlling the connection manager
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
enum ConMgrCommand {
    Connect(Lib3hUri, TcpWss),
    SendData(Lib3hUri, ht::SpanWrap<WsFrame>),
    Disconnect(Lib3hUri),
}

type EvtSend = tokio::sync::mpsc::UnboundedSender<ConMgrEvent>;
type EvtRecv = tokio::sync::mpsc::UnboundedReceiver<ConMgrEvent>;
type CmdSend = tokio::sync::mpsc::UnboundedSender<ConMgrCommand>;
type CmdRecv = tokio::sync::mpsc::UnboundedReceiver<ConMgrCommand>;

pub type ConnectionMgrEventRecv = EvtRecv;

/// internal websocket polling loop
async fn wss_task(
    uri: Lib3hUri,
    mut wss: TcpWss,
    evt_send: EvtSend,
    mut cmd_recv: CmdRecv,
    tracer: Option<ht::Tracer>,
) {
    let mut frame = None;

    // TODO - this should be done with tokio tcp streams && selecting
    //        for now, we're just pausing when no work happens

    'wss_task_loop: loop {
        let mut did_work = false;

        let loop_start = std::time::Instant::now();
        let mut cmd_count = 0;
        let mut read_count = 0;

        // first, process a batch of control commands
        for _ in 0..10 {
            match cmd_recv.try_recv() {
                Ok(cmd) => {
                    cmd_count += 1;
                    did_work = true;
                    match cmd {
                        ConMgrCommand::SendData(_uri, frame) => {
                            let _spanguard = ht::follow(&tracer, &frame, here!(()));
                            if let Err(e) = wss.write(frame.data) {
                                error!("socket write error {} {:?}", uri, e);
                                let _ = evt_send
                                    .send(ConMgrEvent::Disconnect(uri.clone(), Some(e.into())));
                                // end task
                                break 'wss_task_loop;
                            }
                        }
                        ConMgrCommand::Disconnect(_uri) => {
                            debug!("disconnecting socket {}", uri);
                            let _ = evt_send.send(ConMgrEvent::Disconnect(uri.clone(), None));
                            // end task
                            break 'wss_task_loop;
                        }
                        ConMgrCommand::Connect(_, _) => unreachable!(),
                    }
                }
                Err(tokio::sync::mpsc::error::TryRecvError::Empty) => break,
                Err(tokio::sync::mpsc::error::TryRecvError::Closed) => {
                    debug!("socket cmd channel closed {}", uri);
                    let _ = evt_send.send(ConMgrEvent::Disconnect(uri.clone(), None));
                    // channel broken, end task
                    break 'wss_task_loop;
                }
            }
        }

        // next process a batch of incoming websocket frames
        for _ in 0..10 {
            if frame.is_none() {
                frame = Some(WsFrame::default());
            }
            match wss.read(frame.as_mut().unwrap()) {
                Ok(_) => {
                    read_count += 1;
                    did_work = true;
                    let data = frame.take().unwrap();
                    debug!("socket {} read {} bytes", uri, data.as_bytes().len());
                    if let Err(_) = evt_send.send(ConMgrEvent::ReceiveData(uri.clone(), data)) {
                        debug!("socket evt channel closed {}", uri);
                        // end task
                        break 'wss_task_loop;
                    }
                }
                Err(e) if e.would_block() => break,
                Err(e) => {
                    error!("socket read error {} {:?}", uri, e);
                    let _ = evt_send.send(ConMgrEvent::Disconnect(uri.clone(), Some(e.into())));
                    // end task
                    break 'wss_task_loop;
                }
            }
        }

        trace!(
            "wss_task uri {} process {} commands and {} reads in {} ms",
            uri,
            cmd_count,
            read_count,
            loop_start.elapsed().as_millis(),
        );

        // if we did work we might have more work to do,
        // if not, let this task get parked for a time
        if did_work {
            tokio::task::yield_now().await;
        } else {
            tokio::time::delay_for(std::time::Duration::from_millis(5)).await;
        }
    }

    debug!("wss_task ENDING {}", uri);
}

/// internal actually spawn the above wss_task into the tokio runtime
fn spawn_wss_task(
    uri: Lib3hUri,
    wss: TcpWss,
    evt_send: EvtSend,
    tracer: Option<ht::Tracer>,
) -> CmdSend {
    let (cmd_send, cmd_recv) = tokio::sync::mpsc::unbounded_channel();
    tokio::task::spawn(wss_task(uri, wss, evt_send, cmd_recv, tracer));
    cmd_send
}

/// internal result enum for connection mgr processing loop
enum ConMgrResult {
    DidWork,
    NoWork,
    EndTask,
}

use ConMgrResult::*;

/// internal loop for processing the connection mgr
async fn con_mgr_task(mut con_mgr: ConnectionMgr, weak_ref_dummy: Weak<()>) {
    'con_mgr_task: loop {
        if let None = weak_ref_dummy.upgrade() {
            // no more references, let this task end
            break 'con_mgr_task;
        }

        match con_mgr.process() {
            DidWork => tokio::task::yield_now().await,
            NoWork => tokio::time::delay_for(std::time::Duration::from_millis(5)).await,
            EndTask => break 'con_mgr_task,
        }
    }
    warn!("sim2h connection manager task ENDING");
}

/// ConnectionMgr tracks a set of open websocket connections
/// allowing you to send data to them and checking them for incoming data
pub struct ConnectionMgr {
    cmd_recv: CmdRecv,
    evt_send_to_parent: EvtSend,
    evt_send_from_children: EvtSend,
    evt_recv_from_children: EvtRecv,
    connection_count: ConnectionCount,
    wss_map: std::collections::HashMap<Lib3hUri, CmdSend>,
    tracer: Option<ht::Tracer>,
}

impl ConnectionMgr {
    /// spawn a new connection manager task, returning a handle for controlling it
    /// and a receiving channel for any incoming data
    pub fn new(
        tracer: Option<ht::Tracer>,
    ) -> (ConnectionMgrHandle, ConnectionMgrEventRecv, ConnectionCount) {
        let (evt_p_send, evt_p_recv) = tokio::sync::mpsc::unbounded_channel();
        let (evt_c_send, evt_c_recv) = tokio::sync::mpsc::unbounded_channel();
        let (cmd_send, cmd_recv) = tokio::sync::mpsc::unbounded_channel();

        let ref_dummy = Arc::new(());

        let weak_ref_dummy = Arc::downgrade(&ref_dummy);

        let connection_count = ConnectionCount::new();

        let con_mgr = ConnectionMgr {
            cmd_recv,
            evt_send_to_parent: evt_p_send,
            evt_send_from_children: evt_c_send,
            evt_recv_from_children: evt_c_recv,
            connection_count: connection_count.clone(),
            wss_map: std::collections::HashMap::new(),
            tracer,
        };

        tokio::task::spawn(con_mgr_task(con_mgr, weak_ref_dummy));

        (
            ConnectionMgrHandle::new(ref_dummy, cmd_send),
            evt_p_recv,
            connection_count,
        )
    }

    /// internal check our channels
    fn process(&mut self) -> ConMgrResult {
        let mut did_work = false;

        let loop_start = std::time::Instant::now();

        let mut cmd_count = 0;
        let mut recv_count = 0;

        let c_count = self.wss_map.len();

        // first, if any of our handles sent commands / process a batch of them
        for _ in 0..100 {
            match self.cmd_recv.try_recv() {
                Ok(cmd) => {
                    cmd_count += 1;
                    did_work = true;
                    match cmd {
                        ConMgrCommand::SendData(uri, frame) => {
                            let mut remove = false;
                            if let Some(cmd_send) = self.wss_map.get(&uri) {
                                if let Err(_) =
                                    cmd_send.send(ConMgrCommand::SendData(uri.clone(), frame))
                                {
                                    remove = true;
                                }
                            }
                            if remove {
                                self.wss_map.remove(&uri);
                            }
                        }
                        ConMgrCommand::Disconnect(uri) => {
                            if let Some(cmd_send) = self.wss_map.remove(&uri) {
                                let _ = cmd_send.send(ConMgrCommand::Disconnect(uri));
                            }
                        }
                        ConMgrCommand::Connect(uri, wss) => {
                            let cmd_send = spawn_wss_task(
                                uri.clone(),
                                wss,
                                self.evt_send_from_children.clone(),
                                self.tracer.clone(),
                            );
                            if let Some(old) = self.wss_map.insert(uri.clone(), cmd_send) {
                                error!("REPLACING ACTIVE CONNECTION: {}", uri);
                                let _ = old.send(ConMgrCommand::Disconnect(uri));
                            }
                        }
                    }
                }
                Err(tokio::sync::mpsc::error::TryRecvError::Empty) => break,
                Err(tokio::sync::mpsc::error::TryRecvError::Closed) => {
                    // channel broken, end task
                    return EndTask;
                }
            }
        }

        // next, if any of our child wss_tasks sent info, process it
        // mostly we just need to know if any are disconnected.
        // we forward all other messages
        for _ in 0..100 {
            match self.evt_recv_from_children.try_recv() {
                Ok(evt) => {
                    recv_count += 1;
                    did_work = true;
                    match evt {
                        ConMgrEvent::Disconnect(uri, maybe_err) => {
                            if let Some(cmd_send) = self.wss_map.remove(&uri) {
                                let _ = cmd_send.send(ConMgrCommand::Disconnect(uri.clone()));
                            }
                            if let Err(_) = self
                                .evt_send_to_parent
                                .send(ConMgrEvent::Disconnect(uri, maybe_err))
                            {
                                // channel broken, end task
                                return EndTask;
                            }
                        }
                        evt @ _ => {
                            // just forward
                            if let Err(_) = self.evt_send_to_parent.send(evt) {
                                // channel broken, end task
                                return EndTask;
                            }
                        }
                    }
                }
                Err(tokio::sync::mpsc::error::TryRecvError::Empty) => break,
                Err(tokio::sync::mpsc::error::TryRecvError::Closed) => {
                    // channel broken, end task
                    return EndTask;
                }
            }
        }

        let new_c_count = self.wss_map.len();
        if new_c_count != c_count {
            let connection_count = self.connection_count.clone();
            tokio::task::spawn(async move {
                *connection_count.0.write().await = new_c_count;
            });
        }

        trace!(
            "connection_mgr process {} commands and {} recv in {} ms",
            cmd_count,
            recv_count,
            loop_start.elapsed().as_millis(),
        );

        if did_work {
            DidWork
        } else {
            NoWork
        }
    }
}

/// when you create a ConnectionMgr - it is spawned/owned by a tokio task
/// this handle that is returned allows you to send it messages
#[derive(Clone)]
pub struct ConnectionMgrHandle {
    // just kept for reference counting
    _ref_dummy: Arc<()>,
    send_cmd: CmdSend,
}

impl ConnectionMgrHandle {
    /// private constructor - used by ConnectionMgr::new()
    fn new(ref_dummy: Arc<()>, send_cmd: CmdSend) -> Self {
        Self {
            _ref_dummy: ref_dummy,
            send_cmd,
        }
    }

    /// send in a websocket connection to be managed
    pub fn connect(&self, uri: Lib3hUri, wss: TcpWss) {
        if let Err(e) = self.send_cmd.send(ConMgrCommand::Connect(uri, wss)) {
            error!("failed to send on channel - shutting down? {:?}", e);
        }
    }

    /// send data to a managed websocket connection
    pub fn send_data(&self, uri: Lib3hUri, frame: ht::SpanWrap<WsFrame>) {
        if let Err(e) = self.send_cmd.send(ConMgrCommand::SendData(uri, frame)) {
            error!("failed to send on channel - shutting down? {:?}", e);
        }
    }

    /// disconnect and forget about a managed websocket connection
    pub fn disconnect(&self, uri: Lib3hUri) {
        if let Err(e) = self.send_cmd.send(ConMgrCommand::Disconnect(uri)) {
            error!("failed to send on channel - shutting down? {:?}", e);
        }
    }
}
