use crate::*;
use std::sync::Arc;

pub enum ConMgrEvent {
    Disconnect(Lib3hUri, Option<Sim2hError>),
    Receive(Lib3hUri, WsFrame),
}

enum ConMgrCommand {
    Connect(Lib3hUri, TcpWss),
    Disconnect(Lib3hUri),
}

enum ConMgrBoth {
    Cmd(ConMgrCommand),
    Evt(ConMgrEvent),
}

fn spawn_wss_task(uri: Lib3hUri, wss: TcpWss) -> tokio::sync::mpsc::UnboundedSender<ConMgrRecv> {
    let (send_raw_cmd, recv_raw_cmd) = tokio::sync::mpsc::unbounded_channel();

    send_raw_cmd;
}

struct WssMapItem {
    send_raw_cmd: tokio::sync::mpsc::UnboundedSender<ConMgrRecv>,
}

pub struct ConnectionMgr {
    send_event: tokio::sync::mpsc::UnboundedSender<ConMgrEvent>,
    recv_cmd: tokio::sync::mpsc::UnboundedReceiver<ConMgrCommand>,
    wss_map: std::collections::HashMap<Lib3hUri, WssMapItem>,
}

impl ConnectionMgr {
    pub fn new() -> (
        ConnectionMgrHandle,
        tokio::sync::mpsc::UnboundedReceiver<ConMgrEvent>,
    ) {
        let (send_event, recv_event) = tokio::sync::mpsc::unbounded_channel();
        let (send_cmd, recv_cmd) = tokio::sync::mpsc::unbounded_channel();

        let ref_dummy = Arc::new(());

        let weak_ref_dummy = Arc::downgrade(&ref_dummy);

        let con_mgr = ConnectionMgr {
            send_event,
            recv_cmd,
        }

        tokio::task::spawn(async move {
            let (send_raw_event, recv_raw_event) = tokio::sync::mpsc::unbounded_channel();

            // build up a single stream we can efficiently wait on
            let recv_both = {
                let recv_raw_event_both = recv_raw_event.map(|e| ConMgrBoth::Evt(e));
                let recv_cmd_both = recv_cmd.map(|c| ConMgrBoth::Cmd(c));
                recv_cmd_both.merge(recv_raw_event_both)
            };

            loop {
                if let None = weak_ref_dummy.upgrade() {
                    // no more references, let this task end
                    return;
                }

                let msg = match recv_both.next().await {
                    // stream is done, let this task end
                    None => return,
                    Some(msg) => msg,
                };

                match msg {
                    ConMgrBoth::Cmd(cmd) => {
                        match cmd {
                            ConMgrCommand::Connect(uri, wss) => {
                                let send_raw_cmd = spawn_wss_task(
                                    uri.clone(),
                                    wss,
                                );
                                con_mgr.wss_map.insert(uri, WssMapItem {
                                    send_raw_cmd,
                                });
                            }
                            ConMgrCommand::Disconnect(uri) => {
                                con_mgr.wss_map.remove(uri);
                            }
                        }
                    }
                    ConMgrBoth::Evt(evt) => {
                        ConMgrEvent::Disconnect(uri, _maybe_err) => {
                            con_mgr.wss_map.remove(uri);
                        }
                        ConMgrEvent::Receive(_uri, _frame) => {
                            // these should never come here...
                            // only send directly to our owner
                            unreachable!();
                        }
                    }
                }
            }
        });

        (
            ConnectionMgrHandle::new(ref_dummy, send_cmd),
            recv_event,
        )
    }
}

pub struct ConnectionMgrHandle {
    // just kept for reference counting
    _ref_dummy: Arc<()>,
    send_cmd: tokio::sync::mpsc::UnboundedSender<ConMgrCommand>,
}

impl ConnectionMgrHandle {
    pub fn new(
        ref_dummy: Arc<()>,
        send_cmd: tokio::sync::mpsc::UnboundedSender<ConMgrCommand>,
    ) -> Self {
        Self {
            _ref_dummy: ref_dummy,
            send_cmd,
        }
    }

    pub fn connect(&self, uri: Lib3hUri, wss: TcpWss) {
        if let Err(e) = self.send_cmd.send(ConMgrCommand::Connect(uri, wss)) {
            error!("failed to send on channel - shutting down? {:?}", e);
        }
    }

    pub fn disconnect(&self, uri: Lib3hUri) {
        if let Err(e) = self.send_cmd.send(ConMgrCommand::Disconnect(uri)) {
            error!("failed to send on channel - shutting down? {:?}", e);
        }
    }
}
