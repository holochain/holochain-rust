#[cfg(test)]
pub mod tests {

    use crate::sim2h_worker::*;
    use sim2h::*;
    use lib3h_sodium::SodiumCryptoSystem;
    use lib3h_protocol::uri::Builder;
    use test_utils::mock_signing::mock_conductor_api;
    use std::sync::Arc;
    use holochain_locksmith::RwLock as RwLock;
    use holochain_persistence_api::cas::content::AddressableContent;
    use tokio::runtime::current_thread::Runtime;
    use netsim::{Network, node, Ipv4Range};
    use futures::future;
    use futures::sync::oneshot;
    use futures::future::Future;
    use std::thread::sleep;
    use crate::connection::{
        net_connection::{NetHandler, NetWorker},
    };
    use failure::_core::time::Duration;
    use holochain_conductor_lib_api::{ConductorApi};
    use lib3h_protocol::{
        data_types::{SpaceData},
        protocol_client::Lib3hClientProtocol,
        types::{AgentPubKey, SpaceHash},
    };
    use url::Url;

    #[test]
    fn can_connect_to_server() {
        let mut runtime = Runtime::new().unwrap();
        let network = Network::new();
        let network_handle = network.handle();

        runtime.block_on(futures::future::lazy(move || {
            // create a channel to send the client the address of the server
            let (server_addr_tx, server_addr_rx) = oneshot::channel();

            let server_recipe = node::ipv4::machine(|ip| {
                // start up a server node
                let port = 9000;
                let sim2h_url = format!("wss://{}:{}", ip, port);
                let uri = Builder::with_raw_url(Url::parse("wss://0.0.0.0").unwrap())
                    .unwrap_or_else(|e| panic!("with_raw_url: {:?}", e))
                    .with_port(port)
                    .build();
                
                println!("[server] listening on = {}", uri.to_string());
                let _ = server_addr_tx.send(sim2h_url);

                let mut sim2h = Sim2h::new(Box::new(SodiumCryptoSystem::new()), uri);

                loop {
                    match sim2h.process() {
                        Ok(_) => {
                            println!("[server] tick");
                        }
                        Err(e) => {
                            println!("[server] error: {}", e.to_string())
                        }
                    }
                    if false { // keep the compiler happy
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(500));
                }

                future::ok(())
            });

            let client_recipe = node::ipv4::machine(|ip| {
                // start up a client node
                let sim2h_url = server_addr_rx.wait().unwrap();
                let client_config = Sim2hConfig{sim2h_url: sim2h_url.clone()};
                println!("[client] Client server at {} connecting to sim2h server at {}", ip, sim2h_url.clone());
                
                let agent_id = test_utils::mock_signing::registered_test_agent("loose unit");

                let handler = NetHandler::new(Box::new(|message| {
                    println!("[client] got: {:?}", message);
                    Ok(())
                }));
                
                let mut worker = 
                    Sim2hWorker::new(
                        handler.clone(),
                        client_config.clone(),
                        agent_id.clone().address().clone(),
                        ConductorApi::new(Arc::new(RwLock::new(mock_conductor_api(agent_id.clone()))))
                    ).and_then(|w| {
                        println!("[client] Worker successfully started up");
                        Ok(w)
                    }).expect("Could not start worker");
                
                loop {
                    match worker.tick() {
                        Err(e) => println!("[client] Error occured in p2p network module, on tick: {:?}", e),
                        Ok(_) => println!("[client] tick")
                    }
                    sleep(Duration::from_millis(500));

                    let space_data = SpaceData {
                        /// Identifier of this request
                        request_id: String::from("hi"),
                        space_address: SpaceHash::from("SpaceAddress"),
                        agent_id: AgentPubKey::from(agent_id.address().clone()),
                    };

                    worker.receive(Lib3hClientProtocol::JoinSpace(space_data)).unwrap();
                    if false { // keep the compiler happy
                        break;
                    }
                }
                future::ok(())
            });

            let router_recipe = node::ipv4::router((server_recipe, client_recipe));
            let (spawn_complete, _ipv4_plug) = network_handle.spawn_ipv4_tree(Ipv4Range::global(), router_recipe);
            spawn_complete.map(|_| ())

        })).unwrap();
    }
} 
