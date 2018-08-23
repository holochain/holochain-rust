
use holochain_dna::Dna;
use hash_table::entry::Entry;
use std::str::FromStr;
use holochain_agent::Agent;
use holochain_agent::Identity;
use serde_json;

pub trait ToEntry {
  fn to_entry(&self) -> Entry;
  // FIXME - Maybe change to `new_from_entry` ?
  fn from_entry(&Entry) -> Self;
}


//-------------------------------------------------------------------------------------------------
// Entry Type
//-------------------------------------------------------------------------------------------------

// Macro for statically concatanating the system entry prefix for entry types of system entries
macro_rules! sys_prefix {
    ($s:expr) => ( concat!("%", $s) )
}

#[derive(Debug, Clone, PartialEq)]
pub enum EntryType {
  AgentId,
  Deletion,
  Data,
  Dna,
  Headers,
  Key,
  Link,
  Migration,
}

impl FromStr for EntryType {
  type Err = usize;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      sys_prefix!("agent_id") => Ok(EntryType::AgentId),
      sys_prefix!("deletion") => Ok(EntryType::Deletion),
      sys_prefix!("dna") => Ok(EntryType::Dna),
      sys_prefix!("headers") => Ok(EntryType::Headers),
      sys_prefix!("key") => Ok(EntryType::Key),
      sys_prefix!("link") => Ok(EntryType::Link),
      sys_prefix!("migration") => Ok(EntryType::Migration),
      _ => Ok(EntryType::Data),
    }
  }
}

impl EntryType {
  pub fn as_str(&self) -> &'static str {
    match *self {
      EntryType::Data =>      panic!("should not try to convert a custom data entry to str"),
      EntryType::AgentId =>   sys_prefix!("agent_id"),
      EntryType::Deletion =>  sys_prefix!("deletion"),
      EntryType::Dna =>       sys_prefix!("dna"),
      EntryType::Headers =>   sys_prefix!("headers"),
      EntryType::Key =>       sys_prefix!("key"),
      EntryType::Link =>      sys_prefix!("link"),
      EntryType::Migration => sys_prefix!("migration"),
    }
  }
}

//-------------------------------------------------------------------------------------------------
// Dna Entry
//-------------------------------------------------------------------------------------------------

impl ToEntry for Dna {
  fn to_entry(&self) -> Entry {
    Entry::new(
      EntryType::Dna.as_str(),
      &self.to_json().unwrap(),
    )
  }

  fn from_entry(entry: &Entry) -> Self {
    assert!(EntryType::from_str(&entry.entry_type()).unwrap() == EntryType::Dna);
    return Dna::new_from_json(&entry.content())
      .expect("entry is not a valid Dna Entry")
    ;
  }
}


//-------------------------------------------------------------------------------------------------
// Agent Entry
//-------------------------------------------------------------------------------------------------

impl ToEntry for Agent {
  fn to_entry(&self) -> Entry {
    Entry::new(
      EntryType::AgentId.as_str(),
      &self.to_string(),
    )
  }

  fn from_entry(entry: &Entry) -> Self {
    assert!(EntryType::from_str(&entry.entry_type()).unwrap() == EntryType::AgentId);
    let id_content : String = serde_json::from_str(&entry.content())
      .expect("entry is not a valid AgentId Entry");
    Agent::new(Identity::new(id_content))
  }
}


//-------------------------------------------------------------------------------------------------
// UNIT TESTS
//-------------------------------------------------------------------------------------------------

#[cfg(test)]
pub mod tests {
//  extern crate test_utils;
//  extern crate holochain_agent;
//  use super::*;
////  extern crate holochain_core;
////  use holochain_core::{
////    context::Context,
////    nucleus::ribosome::{callback::Callback, Defn},
////    persister::SimplePersister,
////  };
//  use std::sync::{Arc, Mutex};
//  use test_utils::{create_test_dna_with_wasm, create_test_dna_with_wat, create_wasm_from_file};
//
//  use instance::{
//    tests::{test_context, test_instance, test_instance_blank},
//    Instance,
//  };

  // Committing a DnaEntry to source chain should work
  #[test]
  fn can_commit_dna() {
//    // FIXME
//    // Setup the holochain instance
//    let wasm = create_wasm_from_file(
//      "wasm-test/source_chain/target/wasm32-unknown-unknown/debug/source_chain.wasm",
//    );
//    let dna = create_test_dna_with_wasm("test_zome", "test_cap", wasm);
//    let (context, _) = test_context("alex");
//
//
//    let mut hc = Holochain::new(dna.clone(), context).unwrap();
//    // Run the holochain instance
//    hc.start().expect("couldn't start");
//    // Call the exposed wasm function that calls the Commit API function
//    let result = hc.call("test_zome", "test_cap", "can_commit_dna", r#"{}"#);
//    // Expect normal OK result with hash
//    match result {
//      Ok(result) => assert_eq!(
//        result,
//        r#"{"hash":"QmRN6wdp1S2A5EtjW9A3M1vKSBuQQGcgvuhoMUoEz4iiT5"}"#
//      ),
//      Err(_) => assert!(false),
//    };

  }


  // Committing a DnaEntry to source chain should work only if first entry
  #[test]
  fn cannot_commit_dna() {
    // FIXME
  }


  // Committing an AgentIdEntry to source chain should work
  #[test]
  fn can_commit_agent() {
    // FIXME
  }


  // Committing an AgentIdEntry to source chain should work only as second entry
  #[test]
  fn cannot_commit_agent() {
    // FIXME
  }
}