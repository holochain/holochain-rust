use logging::prelude::*;

use super::{
    net_connection::{NetHandler, NetSend, NetShutdown, NetWorkerFactory},
    NetResult,
};
use snowflake::ProcessUniqueId;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    thread, time,
};

use lib3h_protocol::protocol_client::Lib3hClientProtocol;

const TICK_SLEEP_MIN_US: u64 = 100;
const TICK_SLEEP_MAX_US: u64 = 10_000;

/// Struct for holding a network connection running on a separate thread.
/// It is itself a NetSend, and spawns a NetWorker.
pub struct NetConnectionThread {
    can_keep_running: Arc<AtomicBool>,
    send_channel: mpsc::Sender<Lib3hClientProtocol>,
    thread: thread::JoinHandle<()>,
    done: NetShutdown,
    pub endpoint: String,
    pub p2p_endpoint: url::Url,
}

impl NetSend for NetConnectionThread {
    /// send a message to the worker within NetConnectionThread's child thread.
    fn send(&mut self, data: Lib3hClientProtocol) -> NetResult<()> {
        self.send_channel.send(data)?;
        Ok(())
    }
}

impl NetConnectionThread {
    /// NetSendThread Constructor.
    /// Spawns a thread that will create and run a NetWorker with the given factory, handler and
    /// shutdown closure.
    pub fn new(
        handler: NetHandler,
        worker_factory: NetWorkerFactory,
        done: NetShutdown,
    ) -> NetResult<Self> {
        // Create shared bool between self and spawned thread
        let can_keep_running = Arc::new(AtomicBool::new(true));
        let can_keep_running_child = can_keep_running.clone();
        // Create channels between self and spawned thread
        let (send_channel, recv_channel) = mpsc::channel();
        let (send_endpoint, recv_endpoint) = mpsc::channel();

        // Spawn worker thread
        let thread = thread::Builder::new().name(format!("net_worker_thread/{}", ProcessUniqueId::new().to_string())).spawn(move || {
            // Create worker
            let mut worker = worker_factory(handler).expect("able to create worker");
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
                    .try_recv()
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
            // Stop the worker
            worker.stop().unwrap_or_else(|e| {
                error!("Error occured in p2p network module on stop: {:?}", e)
            });
        }).expect("Could not spawn net connection thread");

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
            send_channel,
            thread,
            done,
            endpoint,
            p2p_endpoint,
        })
    }

    /// stop the worker thread (join)
    pub fn stop(self) -> NetResult<()> {
        // tell child thread to stop running
        self.can_keep_running.store(false, Ordering::Relaxed);
        if self.thread.join().is_err() {
            bail!("NetConnectionThread failed to join on stop() call");
        }
        // Call shutdown closure if any
        if let Some(mut done) = self.done {
            done();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{super::net_connection::NetWorker, *};
    use crossbeam_channel::unbounded;
    use holochain_persistence_api::hash::HashString;
    use lib3h_protocol::{data_types::GenericResultData, protocol_server::Lib3hServerProtocol};
    use lib3h_protocol::types::SpaceHash;

    struct DefWorker;

    impl NetWorker for DefWorker {
        fn p2p_endpoint(&self) -> Option<url::Url> {
            Some(url::Url::parse("test://def-worker").unwrap())
        }
    }


    fn success_server_result(result_info: &Vec<u8>) -> Lib3hServerProtocol {
        Lib3hServerProtocol::SuccessResult(GenericResultData {
            request_id: "test_req_id".into(),
            space_address: SpaceHash::from(HashString::from("test_space")),
            to_agent_id: HashString::from("test-agent"),
            result_info : result_info.clone().into(),
        })
    }

    fn success_client_result(result_info: Vec<u8>) -> Lib3hClientProtocol {
        Lib3hClientProtocol::SuccessResult(GenericResultData {
            request_id: "test_req_id".into(),
            space_address: SpaceHash::from(HashString::from("test_space")),
            to_agent_id: HashString::from("test-agent"),
            result_info : result_info.into(),
        })
    }

    #[test]
    fn it_can_defaults() {
        let mut con = NetConnectionThread::new(
            NetHandler::new(Box::new(move |_r| Ok(()))),
            Box::new(|_h| Ok(Box::new(DefWorker) as Box<dyn NetWorker>)),
            None,
        )
        .unwrap();

        con.send(success_client_result("tick".to_string().into_bytes()))
            .unwrap();
        con.stop().unwrap();
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

        fn receive(&mut self, data: Lib3hClientProtocol) -> NetResult<()> {
            match data {
                Lib3hClientProtocol::SuccessResult(data) => self
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
            None,
        )
        .unwrap();

        con.send(success_client_result("test".to_string().into_bytes()))
            .unwrap();

        let res;

        loop {
            let tmp = receiver.recv().unwrap();

            match tmp {
                Lib3hServerProtocol::SuccessResult(generic_data) => {
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

        con.stop().unwrap();
    }

    #[test]
    fn it_can_tick() {
        let (sender, receiver) = unbounded();

        let con = NetConnectionThread::new(
            NetHandler::new(Box::new(move |r| {
                sender.send(r?)?;
                Ok(())
            })),
            Box::new(|h| Ok(Box::new(SimpleWorker { handler: h }) as Box<dyn NetWorker>)),
            None,
        )
        .unwrap();

        let res = receiver.recv().unwrap();

        match res {
            Lib3hServerProtocol::SuccessResult(generic_data) => {
                assert_eq!("tick".to_string().into_bytes(), *generic_data.result_info)
            }
            msg => panic!("unexpected message received: {:?}", msg),
        }
        con.stop().unwrap();
    }
}
