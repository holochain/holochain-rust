use error::DefaultResult;
use holochain_conductor_lib::conductor::Conductor;
use holochain_persistence_api::cas::content::{Address, AddressableContent};
use std::path::Path;

pub fn hash_dna(dna_file_path: &Path) -> DefaultResult<Address> {
    let dna = Conductor::load_dna(dna_file_path)?;
    Ok(dna.address())
}
