use super::{
    net_connection::{NetHandler, NetSend, NetWorkerFactory},
    NetResult,
};
use crate::p2p_network::Lib3hClientProtocolWrapped;
use failure::err_msg;
use holochain_locksmith::Mutex;
use holochain_logging::prelude::*;
use lib3h_protocol::protocol_client::Lib3hClientProtocol;
use snowflake::ProcessUniqueId;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread, time,
};

const TICK_SLEEP_MIN_US: u64 = 100;
const TICK_SLEEP_MAX_US: u64 = 10_000;
const TICK_SLEEP_STARTUP_RETRY_MS: u64 = 3_000;

/// Struct for holding a network connection running on a separate thread.
/// It is itself a NetSend, and spawns a NetWorker.
#[derive(Clone)]
pub struct NetConnectionThread {
    can_keep_running: Arc<AtomicBool>,
    send_channel: ht::channel::EncodedSpanSender<Lib3hClientProtocol>,
    thread: Arc<Mutex<Option<thread::JoinHandle<()>>>>,
    pub endpoint: String,
    pub p2p_endpoint: url::Url,
}

impl NetSend for NetConnectionThread {
    /// send a message to the worker within NetConnectionThread's child thread.
    fn send(&mut self, data: Lib3hClientProtocolWrapped) -> NetResult<()> {
        self.send_channel.send(data)?;
        Ok(())
    }
}

impl NetConnectionThread {
    /// NetSendThread Constructor.
    /// Spawns a thread that will create and run a NetWorker with the given factory, handler and
    /// shutdown closure.
    pub fn new(handler: NetHandler, worker_factory: NetWorkerFactory) -> NetResult<Self> {
        // Create shared bool between self and spawned thread
        let can_keep_running = Arc::new(AtomicBool::new(true));
        let can_keep_running_child = can_keep_running.clone();
        // Create channels between self and spawned thread
        let (send_channel, recv_channel) = crossbeam_channel::unbounded();
        let (send_endpoint, recv_endpoint) = crossbeam_channel::unbounded();

        // Spawn worker thread
        let thread = thread::Builder::new()
            .name(format!(
                "net_worker_thread/{}",
                ProcessUniqueId::new().to_string()
            ))
            .spawn(move || {
                // Try to create a worker. Keep retrying if unsuccessful
                let mut worker = loop {
                    match worker_factory(handler.clone()) {
                        Ok(worker) => {
                            break worker;
                        }
                        Err(e) => {
                            debug!("Error occured in p2p network module, on startup: {:?}", e);
                            debug!(
                                "Waiting {} milliseconds to retry",
                                TICK_SLEEP_STARTUP_RETRY_MS
                            );
                        }
                    }
                    thread::sleep(time::Duration::from_millis(TICK_SLEEP_STARTUP_RETRY_MS));
                };
                // Get endpoint and send it to owner (NetConnectionThread)
                send_endpoint
                    .send((worker.endpoint(), worker.p2p_endpoint()))
                    .expect("Sending endpoint address should work.");
                drop(send_endpoint);
                // Loop as long owner wants to
                let mut sleep_duration_us = TICK_SLEEP_MIN_US;
                while can_keep_running_child.load(Ordering::Relaxed) {
                    // Check if we received something from parent (NetConnectionThread::send())
                    let mut did_something = false;
                    recv_channel
                        .try_recv() // TODO: can we use recv_timeout instead to reduce the poll interval?
                        .and_then(|data| {
                            // Received data from parent
                            // Have the worker handle it
                            did_something = true;
                            worker.receive(data).unwrap_or_else(|e| {
                                debug!("Error occured in p2p network module, on receive: {:?}", e)
                            });
                            Ok(())
                        })
                        .unwrap_or(());
                    // Tick the worker
                    // (it might call the handler if it received a message from the network)
                    worker
                        .tick()
                        .and_then(|b| {
                            if b {
                                did_something = true;
                            }
                            Ok(())
                        })
                        .unwrap_or_else(|e| {
                            error!("Error occured in p2p network module, on tick: {:?}", e)
                        });

                    // Increase sleep duration if nothing was received or sent
                    if did_something {
                        sleep_duration_us = TICK_SLEEP_MIN_US;
                    } else {
                        sleep_duration_us *= 2_u64;
                        if sleep_duration_us > TICK_SLEEP_MAX_US {
                            sleep_duration_us = TICK_SLEEP_MAX_US;
                        }
                    }
                    // Sleep
                    thread::sleep(time::Duration::from_micros(sleep_duration_us));
                }
                debug!("Stopped NetWorker");
                // Stop the worker
                worker.stop().unwrap_or_else(|e| {
                    error!("Error occured in p2p network module on stop: {:?}", e)
                });
            })
            .expect("Could not spawn net connection thread");

        // Retrieve endpoint from spawned thread.
        let (endpoint, p2p_endpoint) = recv_endpoint.recv().map_err(|e| {
            format_err!("Failed to receive endpoint address from net worker: {}", e)
        })?;
        let endpoint = endpoint
            .expect("Should have an endpoint address")
            .to_string();
        let p2p_endpoint = p2p_endpoint.unwrap_or(url::Url::parse("null:").unwrap());

        // Done
        Ok(NetConnectionThread {
            can_keep_running,
            send_channel: send_channel.into(),
            thread: Arc::new(Mutex::new(Some(thread))),
            endpoint,
            p2p_endpoint,
        })
    }

    /// Tell the worker thread to stop, but do not wait for it to join
    pub fn stop(&mut self) {
        debug!("Telling NetWorker to stop");
        self.can_keep_running.store(false, Ordering::Relaxed);
    }

    /// Wait for the worker thread to join (which it may not have done yet when running `stop`)
    #[allow(dead_code)]
    pub fn join_thread(&mut self) -> NetResult<()> {
        if let Some(handle) = self
            .thread
            .lock()
            .map_err(|e| err_msg(format!("Could not get lock on thread handle: {:?}", e)))?
            .take()
        {
            handle.join().map_err(|e| {
                err_msg(format!(
                    "NetConnectionThread failed to join on stop() call: {:?}",
                    e
                ))
            })?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{super::net_connection::NetWorker, *};
    use crate::p2p_network::Lib3hServerProtocolWrapped;
    use crossbeam_channel::unbounded;
    use holochain_persistence_api::hash::HashString;
    use lib3h_protocol::{
        data_types::GenericResultData,
        types::{AgentPubKey, SpaceHash},
    };

    struct DefWorker;

    impl NetWorker for DefWorker {
        fn p2p_endpoint(&self) -> Option<url::Url> {
            Some(url::Url::parse("test://def-worker").unwrap())
        }
    }

    fn success_server_result(result_info: &Vec<u8>) -> Lib3hServerProtocolWrapped {
        Lib3hServerProtocolWrapped::SuccessResult(GenericResultData {
            request_id: "test_req_id".into(),
            space_address: SpaceHash::from(HashString::from("test_space")),
            to_agent_id: AgentPubKey::from("test-agent"),
            result_info: result_info.clone().into(),
        })
    }

    fn success_client_result(result_info: Vec<u8>) -> Lib3hClientProtocolWrapped {
        Lib3hClientProtocolWrapped::SuccessResult(GenericResultData {
            request_id: "test_req_id".into(),
            space_address: SpaceHash::from(HashString::from("test_space")),
            to_agent_id: AgentPubKey::from("test-agent"),
            result_info: result_info.into(),
        })
    }

    #[test]
    fn it_can_defaults() {
        let mut con = NetConnectionThread::new(
            NetHandler::new(Box::new(move |_r| Ok(()))),
            Box::new(|_h| Ok(Box::new(DefWorker) as Box<dyn NetWorker>)),
        )
        .unwrap();

        con.send(success_client_result("tick".to_string().into_bytes()))
            .unwrap();
        con.stop();
    }

    struct SimpleWorker {
        handler: NetHandler,
    }

    impl NetWorker for SimpleWorker {
        fn tick(&mut self) -> NetResult<bool> {
            self.handler
                .handle(Ok(success_server_result(&"tick".to_string().into_bytes())))?;
            Ok(true)
        }

        fn receive(&mut self, data: Lib3hClientProtocolWrapped) -> NetResult<()> {
            match data {
                Lib3hClientProtocolWrapped::SuccessResult(data) => self
                    .handler
                    .handle(Ok(success_server_result(&*data.result_info))),
                msg => panic!("unexpected client protocol message in receive: {:?}", msg),
            }
        }

        fn p2p_endpoint(&self) -> Option<url::Url> {
            Some(url::Url::parse("test://simple-worker").unwrap())
        }
    }

    #[test]
    fn it_invokes_connection_thread() {
        let (sender, receiver) = unbounded();

        let mut con = NetConnectionThread::new(
            NetHandler::new(Box::new(move |r| {
                sender.send(r?)?;
                Ok(())
            })),
            Box::new(|h| Ok(Box::new(SimpleWorker { handler: h }) as Box<dyn NetWorker>)),
        )
        .unwrap();

        con.send(success_client_result("test".to_string().into_bytes()))
            .unwrap();

        let res;

        loop {
            let tmp = receiver.recv().unwrap();

            match tmp {
                Lib3hServerProtocolWrapped::SuccessResult(generic_data) => {
                    if generic_data.result_info == "tick".to_string().into_bytes().into() {
                        continue;
                    } else {
                        res = generic_data.result_info;
                        break;
                    }
                }
                msg => panic!("unexpected message received: {:?}", msg),
            }
        }

        assert_eq!("test".to_string().into_bytes(), *res);

        con.stop();
    }

    #[test]
    fn it_can_tick() {
        let (sender, receiver) = unbounded();

        let mut con = NetConnectionThread::new(
            NetHandler::new(Box::new(move |r| {
                sender.send(r?)?;
                Ok(())
            })),
            Box::new(|h| Ok(Box::new(SimpleWorker { handler: h }) as Box<dyn NetWorker>)),
        )
        .unwrap();

        let res = receiver.recv().unwrap();

        match res {
            Lib3hServerProtocolWrapped::SuccessResult(generic_data) => {
                assert_eq!("tick".to_string().into_bytes(), *generic_data.result_info)
            }
            msg => panic!("unexpected message received: {:?}", msg),
        }
        con.stop();
    }
}
