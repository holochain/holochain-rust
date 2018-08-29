
use serde_json;
use hash_table::sys_entry::{
  EntryType, ToEntry,
};
use hash_table::entry::Entry;
use hash_table::HashString;
use std::str::FromStr;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DeletionEntry {
  entry_hash: HashString,
  message: String,
}

impl DeletionEntry {
  pub fn new(
    entry_hash: &str,
    message: &str,
  ) -> Self {
    DeletionEntry {
      entry_hash: entry_hash.to_string(),
      message: message.to_string(),
    }
  }
}


impl ToEntry for DeletionEntry {
  // Convert a LinkEntry into a JSON array of Links
  fn to_entry(&self) -> Entry {
    let json_array = serde_json::to_string(self).expect("Link should serialize");
    Entry::new(EntryType::Deletion.as_str(), &json_array)
  }

  fn new_from_entry(entry: &Entry) -> Self {
    assert!(EntryType::from_str(&entry.entry_type()).unwrap() == EntryType::Deletion);
    //let content: DeletionEntry =
      return serde_json::from_str(&entry.content()).expect("entry is not a valid Link Entry");
    //DeletionEntry::new(&content)
  }
}



#[cfg(test)]
pub mod tests {
  extern crate test_utils;


  use super::*;
  use action::{Action, ActionWrapper};
  use hash_table::sys_entry::{EntryType, ToEntry};
  use std::str::FromStr;

  use instance::{tests::test_context, Instance, Observer};
  use std::sync::mpsc::channel;


  /// Committing a LinkEntry to source chain should work
  #[test]
  fn can_commit_deletion() {
    // Create Context, Agent, Dna, and Commit AgentIdEntry Action
    let context = test_context("alex");
    let del_entry = DeletionEntry::new("0x42", "test-entry");
    let commit_action = ActionWrapper::new(Action::Commit(del_entry.to_entry()));

    // Set up instance and process the action
    let instance = Instance::new();
    let state_observers: Vec<Observer> = Vec::new();
    let (_, rx_observer) = channel::<Observer>();
    instance.process_action(commit_action, state_observers, &rx_observer, &context);

    // Check if LinkEntry is found
    assert_eq!(1, instance.state().history.iter().count());
    instance
      .state()
      .history
      .iter()
      .find(|aw| match aw.action() {
        Action::Commit(entry) => {
          assert_eq!(
            EntryType::from_str(&entry.entry_type()).unwrap(),
            EntryType::Deletion,
          );
          assert_eq!(entry.content(), del_entry.to_entry().content());
          true
        }
        _ => false,
      });
  }
}