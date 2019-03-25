use crate::{
    conductor::base::KeyLoader,
    keystore::{Keystore, Secret, PRIMARY_KEYBUNDLE_ID},
};
use conductor::passphrase_manager::{PassphraseManager, PassphraseServiceMock};
use holochain_core_types::error::HolochainError;
use holochain_dpki::SEED_SIZE;
use holochain_sodium::{hash::sha256, secbuf::SecBuf};
use keystore::test_hash_config;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

/// Key loader callback to use with conductor_api.
/// This replaces filesystem access for getting keys mentioned in the config.
/// Uses `test_keybundle` to create a deterministic key dependent on the (virtual) file name.
pub fn test_keystore_loader() -> KeyLoader {
    let loader = Box::new(|path: &PathBuf, _pm: Arc<PassphraseManager>| {
        Ok(test_keystore(&path.to_str().unwrap().to_string()))
    })
        as Box<
            FnMut(&PathBuf, Arc<PassphraseManager>) -> Result<Keystore, HolochainError>
                + Send
                + Sync,
        >;
    Arc::new(loader)
}

/// Create a deterministic test key from the SHA256 of the given name string.
pub fn test_keystore(agent_name: &String) -> Keystore {
    let mut keystore = Keystore::new(
        mock_passphrase_manager(agent_name.clone()),
        test_hash_config(),
    )
    .unwrap();

    // Create seed from name
    let mut name = SecBuf::with_insecure_from_string(agent_name.clone());
    let mut seed = SecBuf::with_insecure(SEED_SIZE);
    sha256(&mut name, &mut seed).expect("Could not hash test agent name");

    let secret = Arc::new(Mutex::new(Secret::Seed(seed)));
    keystore.add("root_seed", secret).unwrap();

    keystore
        .add_keybundle_from_seed("root_seed", PRIMARY_KEYBUNDLE_ID)
        .unwrap();
    keystore
}

pub fn mock_passphrase_manager(passphrase: String) -> Arc<PassphraseManager> {
    Arc::new(PassphraseManager::new(Arc::new(Mutex::new(
        PassphraseServiceMock { passphrase },
    ))))
}
