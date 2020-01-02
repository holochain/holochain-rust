#[macro_use]
extern crate url2;

use holochain_conductor_lib_api::ConductorApi;
use holochain_locksmith::{Mutex, RwLock};
use holochain_net::{
    connection::net_connection::{NetHandler, NetWorker},
    sim2h_worker::{Sim2hConfig, Sim2hWorker},
};
use jsonrpc_core::IoHandler;
use lib3h_crypto_api::CryptoSystem;
use lib3h_protocol::{
    data_types::*, protocol_client::Lib3hClientProtocol, protocol_server::Lib3hServerProtocol,
    uri::Lib3hUri,
};
use lib3h_sodium::SodiumCryptoSystem;
use sim2h::Sim2h;
use std::sync::Arc;

#[test]
fn sim2h_worker_talks_to_sim2h() {
    let crypto = Box::new(SodiumCryptoSystem::new());

    let mut pub_key = crypto.buf_new_insecure(crypto.sign_public_key_bytes());
    let mut sec_key = crypto.buf_new_secure(crypto.sign_secret_key_bytes());
    crypto.sign_keypair(&mut pub_key, &mut sec_key).unwrap();

    let enc = hcid::HcidEncoding::with_kind("hcs0").unwrap();
    let agent_id = enc.encode(&*pub_key).unwrap();

    let (snd, rcv) = crossbeam_channel::unbounded();
    let cont = Arc::new(Mutex::new(true));

    let srv_cont = cont.clone();
    let sim2h_join = std::thread::spawn(move || {
        let url = url2!("ws://127.0.0.1:0");
        let mut sim2h = Sim2h::new(
            Box::new(SodiumCryptoSystem::new()),
            Lib3hUri(url.into()),
            false,
        );

        snd.send(sim2h.bound_uri.clone().unwrap()).unwrap();
        drop(snd);

        while *srv_cont.lock().unwrap() {
            if let Err(e) = sim2h.process() {
                panic!("{:?}", e);
            }

            std::thread::yield_now();
        }
    });

    let bound_uri = rcv.recv().unwrap();
    println!("GOT BOUND: {:?}", bound_uri);

    // -- beg sim2h worker test -- //

    let io = Arc::new(RwLock::new(IoHandler::new()));

    let sec_key = Arc::new(Mutex::new(sec_key.box_clone()));
    io.write().unwrap().add_method(
        "agent/sign",
        move |params: jsonrpc_core::types::params::Params| {
            let params = match params {
                jsonrpc_core::types::params::Params::Map(m) => m,
                _ => panic!("bad type"),
            };
            let payload =
                Box::new(base64::decode(params.get("payload").unwrap().as_str().unwrap()).unwrap());
            let mut payload2 = crypto.buf_new_insecure(payload.len());
            payload2.write(0, &payload).unwrap();

            let mut sig = crypto.buf_new_insecure(crypto.sign_bytes());
            crypto.randombytes_buf(&mut sig).unwrap();

            crypto
                .sign(&mut sig, &payload2, &*sec_key.lock().unwrap())
                .unwrap();
            let signature = base64::encode(&*sig.read_lock());
            Ok(serde_json::json!({ "signature": signature }))
        },
    );

    struct ResultData {
        pub got_handle_store: bool,
        pub got_handle_dm: bool,
        pub got_handle_a_list: bool,
        pub got_handle_g_list: bool,
    }

    impl Default for ResultData {
        fn default() -> Self {
            Self {
                got_handle_store: false,
                got_handle_dm: false,
                got_handle_a_list: false,
                got_handle_g_list: false,
            }
        }
    }

    impl ResultData {
        pub fn new() -> Arc<Mutex<Self>> {
            Arc::new(Mutex::new(Self::default()))
        }

        pub fn as_mut(s: &Arc<Mutex<Self>>) -> holochain_locksmith::MutexGuard<Self> {
            s.lock().unwrap()
        }

        pub fn is_ok(s: &Arc<Mutex<Self>>) -> bool {
            let s = s.lock().unwrap();
            s.got_handle_store && s.got_handle_dm && s.got_handle_a_list && s.got_handle_g_list
        }
    }

    let result_data = ResultData::new();

    let result_data_worker = result_data.clone();
    let mut worker = Sim2hWorker::new(
        NetHandler::new(Box::new(move |msg| {
            match msg.unwrap() {
                Lib3hServerProtocol::HandleGetAuthoringEntryList(info) => {
                    println!("HANDLE A LIST: {:?}", info);
                    ResultData::as_mut(&result_data_worker).got_handle_a_list = true;
                }
                Lib3hServerProtocol::HandleGetGossipingEntryList(info) => {
                    println!("HANDLE G LIST: {:?}", info);
                    ResultData::as_mut(&result_data_worker).got_handle_g_list = true;
                }
                Lib3hServerProtocol::HandleStoreEntryAspect(info) => {
                    println!("HANDLE STORE: {:?}", info);
                    ResultData::as_mut(&result_data_worker).got_handle_store = true;
                }
                Lib3hServerProtocol::HandleSendDirectMessage(info) => {
                    println!("HANDLE DM: {:?}", info);
                    ResultData::as_mut(&result_data_worker).got_handle_dm = true;
                }
                e @ _ => panic!("unexpected: {:#?}", e),
            }
            Ok(())
        })),
        Sim2hConfig {
            sim2h_url: bound_uri.as_str().to_string(),
        },
        agent_id.clone().into(),
        ConductorApi::new(io.clone()),
    )
    .unwrap();
    worker.set_full_sync(true);

    worker
        .receive(Lib3hClientProtocol::JoinSpace(SpaceData {
            agent_id: agent_id.clone().into(),
            request_id: "".to_string(),
            space_address: "BLA".to_string().into(),
        }))
        .unwrap();

    worker
        .receive(Lib3hClientProtocol::PublishEntry(ProvidedEntryData {
            space_address: "BLA".to_string().into(),
            provider_agent_id: agent_id.clone().into(),
            entry: EntryData {
                entry_address: "BLA".to_string().into(),
                aspect_list: vec![EntryAspectData {
                    aspect_address: "BLA".to_string().into(),
                    type_hint: "".to_string(),
                    aspect: b"BLA".to_vec().into(),
                    publish_ts: 0,
                }],
            },
        }))
        .unwrap();

    worker
        .receive(Lib3hClientProtocol::SendDirectMessage(DirectMessageData {
            space_address: "BLA".to_string().into(),
            request_id: "".to_string(),
            to_agent_id: agent_id.clone().into(),
            from_agent_id: agent_id.clone().into(),
            content: b"BLA".to_vec().into(),
        }))
        .unwrap();

    for _ in 0..40 {
        std::thread::sleep(std::time::Duration::from_millis(25));

        println!("tick: {:?}", worker.tick());

        if ResultData::is_ok(&result_data) {
            break;
        }
    }

    // -- end sim2h worker test -- //

    *cont.lock().unwrap() = false;
    sim2h_join.join().unwrap();

    assert!(ResultData::is_ok(&result_data));
}
