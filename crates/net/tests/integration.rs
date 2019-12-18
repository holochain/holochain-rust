#[macro_use]
extern crate url2;

use lib3h_crypto_api::CryptoSystem;
use jsonrpc_core::IoHandler;
use holochain_conductor_lib_api::ConductorApi;
use lib3h_protocol::{
    data_types::*,
    protocol_client::Lib3hClientProtocol,
    uri::Lib3hUri,
};
use lib3h_sodium::SodiumCryptoSystem;
use sim2h::Sim2h;
use holochain_net::{
    connection::net_connection::{NetHandler, NetWorker},
    sim2h_worker::{
        Sim2hConfig,
        Sim2hWorker,
    },
};

#[test]
fn sim2h_worker_talks_to_sim2h() {
    let crypto = Box::new(SodiumCryptoSystem::new());

    let mut pub_key = crypto.buf_new_insecure(crypto.sign_public_key_bytes());
    let mut sec_key = crypto.buf_new_secure(crypto.sign_secret_key_bytes());
    crypto.sign_keypair(&mut pub_key, &mut sec_key).unwrap();

    let enc = hcid::HcidEncoding::with_kind("hcs0").unwrap();
    let agent_id = enc.encode(&*pub_key).unwrap();

    let (snd, rcv) = crossbeam_channel::unbounded();
    let cont = std::sync::Arc::new(std::sync::Mutex::new(true));

    let srv_cont = cont.clone();
    let sim2h_join = std::thread::spawn(move || {
        let url = url2!("wss://127.0.0.1:0");
        let mut sim2h = Sim2h::new(Box::new(SodiumCryptoSystem::new()), Lib3hUri(url.into()));

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

    let io = std::sync::Arc::new(holochain_locksmith::RwLock::new(IoHandler::new()));

    let sec_key = std::sync::Arc::new(holochain_locksmith::Mutex::new(sec_key.box_clone()));
    io.write().unwrap().add_method("agent/sign", move |params: jsonrpc_core::types::params::Params| {
        let params = match params {
            jsonrpc_core::types::params::Params::Map(m) => m,
            _ => panic!("bad type"),
        };
        let payload = Box::new(base64::decode(params.get("payload").unwrap().as_str().unwrap()).unwrap());
        let mut payload2 = crypto.buf_new_insecure(payload.len());
        payload2.write(0, &payload).unwrap();

        let mut sig = crypto.buf_new_insecure(crypto.sign_bytes());
        crypto.randombytes_buf(&mut sig).unwrap();

        crypto.sign(&mut sig, &payload2, &*sec_key.lock().unwrap()).unwrap();
        let signature = base64::encode(&*sig.read_lock());
        Ok(serde_json::json!({ "signature": signature }))
    });

    let mut worker = Sim2hWorker::new(
        NetHandler::new(Box::new(move |msg| {
            println!("HANDLE MSG: {:?}", msg);
            Ok(())
        })),
        Sim2hConfig {
            sim2h_url: bound_uri.as_str().to_string(),
        },
        agent_id.clone().into(),
        ConductorApi::new(io.clone()),
    ).unwrap();

    worker.receive(Lib3hClientProtocol::JoinSpace(SpaceData {
        agent_id: agent_id.clone().into(),
        request_id: "".to_string(),
        space_address: "BLA".to_string().into(),
    })).unwrap();

    for _ in 0..10 {
        std::thread::sleep(std::time::Duration::from_millis(100));

        println!("tick: {:?}", worker.tick());
    }

    // -- end sim2h worker test -- //

    *cont.lock().unwrap() = false;
    sim2h_join.join().unwrap();
}
