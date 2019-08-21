use crate::util::get_secure_string_double_check;
use holochain_core_types::error::HcResult;
use holochain_dpki::{
    seed::{RootSeed, SeedTrait},
    utils::generate_random_seed_buf,
};

pub fn dpki_init(_passphrase: Option<String>) -> HcResult<String> {
    let seed_buf = generate_random_seed_buf();
    let mut root_seed = RootSeed::new(seed_buf);

    // prompt for a passphrase to encrypt the root seed.
    // TODO: Actually encrypt to root seed. Passphrase is not used at this time
    let _passphrase = _passphrase.unwrap_or_else(|| {
        get_secure_string_double_check("Root Key Encryption Passphrase (placeholder)", false)
            .expect("Could not obtain passphrase")
    });

    root_seed.seed_mut().get_mnemonic()
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_dpki_init_smoke() {
        assert!(dpki_init(Some("dummy passphrase".to_string())).is_ok(),)
    }
}
