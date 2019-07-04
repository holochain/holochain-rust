use crate::{
    action::{Action, ActionWrapper},
    network::state::NetworkState,
    state::State,
};
use holochain_net::connection::{
    json_protocol::{JsonProtocol, TrackDnaData},
    net_connection::NetSend,
};
use std::{thread::sleep, time::Duration};

pub fn reduce_shutdown(
    state: &mut NetworkState,
    _root_state: &State,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    assert_eq!(*action, Action::ShutdownNetwork);

    let json = JsonProtocol::UntrackDna(TrackDnaData {
        dna_address: state
            .dna_address
            .as_ref()
            .expect("Tried to shutdown uninitialized network")
            .clone(),
        agent_id: state
            .agent_id
            .as_ref()
            .expect("Tried to shutdown uninitialized network")
            .clone()
            .into(),
    });

    let mut network_lock = state.network.lock().unwrap();

    {
        let network = network_lock
            .as_mut()
            .expect("Tried to shutdown uninitialized network");
        let _ = network.send(json.into());
        sleep(Duration::from_secs(2));
    }

    if let Err(err) = network_lock.take().unwrap().stop() {
        println!("ERROR stopping network thread: {:?}", err);
    } else {
        println!("Network thread successfully stopped");
    }
}

#[cfg(test)]
pub mod test {
    use self::tempfile::tempdir;
    use super::*;
    use crate::{
        context::Context,
        logger::test_logger,
        persister::SimplePersister,
        state::{test_store, StateWrapper},
    };
    use holochain_core_types::agent::AgentId;
    use holochain_net::{connection::net_connection::NetHandler, p2p_config::P2pConfig};
    use holochain_persistence_api::cas::content::{Address, AddressableContent};
    use holochain_persistence_file::{cas::file::FilesystemStorage, eav::file::EavFileStorage};
    use std::sync::{Mutex, RwLock};
    use tempfile;

    fn test_context() -> Arc<Context> {
        let file_storage = Arc::new(RwLock::new(
            FilesystemStorage::new(tempdir().unwrap().path().to_str().unwrap()).unwrap(),
        ));
        let mut context = Context::new(
            AgentId::generate_fake("Terence"),
            test_logger(),
            Arc::new(Mutex::new(SimplePersister::new(file_storage.clone()))),
            file_storage.clone(),
            file_storage.clone(),
            Arc::new(RwLock::new(
                EavFileStorage::new(tempdir().unwrap().path().to_str().unwrap().to_string())
                    .unwrap(),
            )),
            P2pConfig::new_with_unique_memory_backend(),
            None,
            None,
        );

        let global_state = Arc::new(RwLock::new(StateWrapper::new(Arc::new(context.clone()))));
        context.set_state(global_state.clone());
        Arc::new(context)
    }

}
