extern crate toml;
extern crate serde_derive;
use holochain_agent::Agent;
use holochain_core_types::error::{HolochainError,HcResult};
use serde::Deserialize;



fn load<'a,T>(toml:&'a str) -> HcResult<T> where T:Deserialize<'a>
{
    toml::from_str::<'a,T>(toml).map_err(|e|{
        HolochainError::IoError(String::from("Could not serialize toml"))
    })
}
