use error::DefaultResult;
use failure::err_msg;
use holochain_conductor_lib::conductor::Conductor;
use holochain_persistence_api::cas::content::{Address, AddressableContent};
use serde_json::Map;
use std::path::PathBuf;

pub fn hash_dna(
    dna_file_path: &PathBuf,
    properties: Option<Vec<String>>,
) -> DefaultResult<Address> {
    let mut dna = Conductor::load_dna(dna_file_path)?;
    let mut map = if let serde_json::Value::Object(map) = dna.properties {
        map.clone()
    } else {
        Map::new()
    };

    if let Some(properties) = properties {
        for property_string in properties {
            let mut parts = property_string
                .split('=')
                .map(String::from)
                .collect::<Vec<String>>();
            if parts.len() != 2 {
                return Err(err_msg(format!(
                    "Can't parse property: {}",
                    property_string
                )));
            }
            let value = parts.pop().unwrap();
            let name = parts.pop().unwrap();
            map.insert(name, serde_json::Value::String(value));
        }
    }

    dna.properties = serde_json::Value::Object(map);
    Ok(dna.address())
}
