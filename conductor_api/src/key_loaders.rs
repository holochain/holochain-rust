use crate::conductor::base::KeyLoader;
use holochain_core_types::error::HolochainError;
use holochain_dpki::{
    key_bundle::{KeyBundle, SeedType},
    utils::SEED_SIZE,
};
use holochain_sodium::{hash::sha256, secbuf::SecBuf};
use std::{path::PathBuf, sync::Arc};

/// Key loader callback to use with conductor_api.
/// This replaces filesystem access for getting keys mentioned in the config.
/// Uses `test_keybundle` to create a deterministic key dependent on the (virtual) file name.
pub fn test_keybundle_loader() -> KeyLoader {
    let loader = Box::new(|path: &PathBuf| Ok(test_keybundle(&path.to_str().unwrap().to_string())))
        as Box<FnMut(&PathBuf) -> Result<KeyBundle, HolochainError> + Send + Sync>;
    Arc::new(loader)
}

/// Create a deterministic test key from the SHA256 of the given name string.
pub fn test_keybundle(name: &String) -> KeyBundle {
    // Create seed from name
    let mut name = SecBuf::with_insecure_from_string(name.clone());
    let mut seed = SecBuf::with_insecure(SEED_SIZE);
    sha256(&mut name, &mut seed).expect("Could not hash test agent name");

    // Create KeyBundle from seed
    KeyBundle::new_from_seed(&mut seed, SeedType::Mock).unwrap()
}
