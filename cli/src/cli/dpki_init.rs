use holochain_core_types::error::HcResult;
use holochain_dpki::{
	seed::{RootSeed, SeedTrait},
	utils::generate_random_seed_buf,
};

pub fn dpki_init() -> HcResult<String> {
	let seed_buf = generate_random_seed_buf();
    let mut root_seed = RootSeed::new(seed_buf);
    root_seed.seed_mut().get_mnemonic()
}

#[cfg(test)]
pub mod tests {
	use super::*;

	#[test]
	fn test_dpki_init_smoke() {
		assert!(
			dpki_init().is_ok(),
		)
	}
}
