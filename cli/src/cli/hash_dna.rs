use error::DefaultResult;
use holochain_conductor_api::conductor::Conductor;
use holochain_persistence_api::cas::content::{Address, AddressableContent};
use std::path::PathBuf;

pub fn hash_dna(dna_file_path: &PathBuf) -> DefaultResult<Address> {
    let dna = Conductor::load_dna(dna_file_path)?;
    Ok(dna.address())
}
