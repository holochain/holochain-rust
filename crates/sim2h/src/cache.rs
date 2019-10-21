//! implements caching structures for spaces and aspects
use crate::AgentId;
use lib3h_protocol::{
    types::{AspectHash, EntryHash},
    uri::Lib3hUri,
};
use std::collections::{HashMap, HashSet};

pub struct Space {
    agents: HashMap<AgentId, Lib3hUri>,
    all_aspects_hashes: AspectList,
}

impl Space {
    pub fn new() -> Self {
        Space {
            agents: HashMap::new(),
            all_aspects_hashes: AspectList::from(HashMap::new()),
        }
    }

    pub fn join_agent(&mut self, agent_id: AgentId, uri: Lib3hUri) {
        self.agents.insert(agent_id, uri);
    }

    pub fn remove_agent(&mut self, agent_id: &AgentId) {
        self.agents.remove(agent_id);
    }

    pub fn agent_id_to_uri(&self, agent_id: &AgentId) -> Option<Lib3hUri> {
        for (found_agent, uri) in self.agents.iter() {
            if found_agent == agent_id {
                return Some(uri.clone());
            }
        }
        None
    }

    pub fn all_agents(&self) -> &HashMap<AgentId, Lib3hUri> {
        &self.agents
    }

    pub fn all_aspects(&self) -> &AspectList {
        &self.all_aspects_hashes
    }

    pub fn add_aspect(&mut self, entry_address: EntryHash, aspect_address: AspectHash) {
        self.all_aspects_hashes.add(entry_address, aspect_address);
    }
}

#[derive(Debug)]
pub struct AspectList(HashMap<EntryHash, Vec<AspectHash>>);
impl AspectList {
    /// Returns an AspectList list that contains every entry aspect
    /// in self that is not in other.
    pub fn diff(&self, other: &AspectList) -> AspectList {
        let self_set = HashSet::<(EntryHash, AspectHash)>::from(self);
        let other_set = HashSet::<(EntryHash, AspectHash)>::from(other);
        AspectList::from(
            &self_set
                .difference(&other_set)
                .cloned()
                .collect::<HashSet<(EntryHash, AspectHash)>>(),
        )
    }

    pub fn add(&mut self, entry_address: EntryHash, aspect_address: AspectHash) {
        self.0
            .entry(entry_address)
            .or_insert_with(Vec::new)
            .push(aspect_address);
    }

    pub fn entry_addresses(&self) -> impl Iterator<Item = &EntryHash> {
        self.0.keys()
    }

    pub fn per_entry(&self, entry_address: &EntryHash) -> Option<&Vec<AspectHash>> {
        self.0.get(entry_address)
    }

    pub fn pretty_string(&self) -> String {
        self.0
            .iter()
            .map(|(entry, aspects)| {
                format!(
                    "{}: [{}]",
                    entry,
                    aspects
                        .iter()
                        .cloned()
                        .map(|i| i.into())
                        .collect::<Vec<String>>()
                        .join(", ")
                )
            })
            .collect::<Vec<String>>()
            .join("\n")
    }
}

impl From<HashMap<EntryHash, Vec<AspectHash>>> for AspectList {
    fn from(map: HashMap<EntryHash, Vec<AspectHash>>) -> AspectList {
        AspectList { 0: map }
    }
}

impl From<&AspectList> for HashSet<(EntryHash, AspectHash)> {
    fn from(a: &AspectList) -> HashSet<(EntryHash, AspectHash)> {
        let mut result = HashSet::new();
        for (entry_address, aspect_list) in a.0.iter() {
            for aspect_address in aspect_list {
                result.insert((entry_address.clone(), aspect_address.clone()));
            }
        }
        result
    }
}

impl From<&HashSet<(EntryHash, AspectHash)>> for AspectList {
    fn from(s: &HashSet<(EntryHash, AspectHash)>) -> AspectList {
        let mut result: HashMap<EntryHash, Vec<AspectHash>> = HashMap::new();
        for (entry_address, aspect_address) in s {
            if !result.contains_key(entry_address) {
                result.insert(entry_address.clone(), Vec::new());
            }
            result
                .get_mut(entry_address)
                .unwrap()
                .push(aspect_address.clone());
        }
        AspectList::from(result)
    }
}
