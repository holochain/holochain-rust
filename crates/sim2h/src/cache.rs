//! implements caching structures for spaces and aspects
use crate::{
    error::*,
    naive_sharding::{entry_location, naive_sharding_should_store},
    AgentId,
};
use im::{HashMap, HashSet};
use lib3h::rrdht_util::*;
use lib3h_crypto_api::CryptoSystem;
use lib3h_protocol::{
    types::{AspectHash, EntryHash},
    uri::Lib3hUri,
};

#[derive(Debug, Clone)]
pub(crate) struct AgentInfo {
    pub uri: Lib3hUri,
    pub location: Location,
}

pub struct Space {
    crypto: Box<dyn CryptoSystem>,
    agents: HashMap<AgentId, AgentInfo>,
    all_aspects_hashes: AspectList,
    missing_aspects: HashMap<AgentId, HashMap<EntryHash, HashSet<AspectHash>>>,
}

impl std::fmt::Debug for Space {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Space")
            .field("agents", &self.agents)
            .field("all_aspect_hashes", &self.all_aspects_hashes)
            .field("missing_aspects", &self.missing_aspects)
            .finish()
    }
}

impl Clone for Space {
    fn clone(&self) -> Self {
        Self {
            crypto: self.crypto.box_clone(),
            agents: self.agents.clone(),
            all_aspects_hashes: self.all_aspects_hashes.clone(),
            missing_aspects: self.missing_aspects.clone(),
        }
    }
}

impl Space {
    pub fn new(crypto: Box<dyn CryptoSystem>) -> Self {
        Space {
            crypto,
            agents: HashMap::new(),
            all_aspects_hashes: AspectList::from(HashMap::new()),
            missing_aspects: HashMap::new(),
        }
    }

    pub fn add_missing_aspect(
        &mut self,
        agent: AgentId,
        entry_hash: EntryHash,
        aspect_hash: AspectHash,
    ) {
        let map_for_agent = self
            .missing_aspects
            .entry(agent)
            .or_insert_with(HashMap::new);
        let hash_set_for_entry = map_for_agent.entry(entry_hash).or_insert_with(HashSet::new);
        hash_set_for_entry.insert(aspect_hash);
    }

    pub fn remove_missing_aspect(
        &mut self,
        agent: &AgentId,
        entry_hash: &EntryHash,
        aspect_hash: &AspectHash,
    ) {
        let maybe_map_for_agent = self.missing_aspects.get_mut(agent);
        if let Some(map_for_agent) = maybe_map_for_agent {
            if let Some(hash_set_for_entry) = map_for_agent.get_mut(entry_hash) {
                hash_set_for_entry.remove(aspect_hash);
                if hash_set_for_entry.len() == 0 {
                    map_for_agent.remove(entry_hash);
                }
            }
            if map_for_agent.len() == 0 {
                self.missing_aspects.remove(agent);
            }
        }
    }

    pub fn agents_with_missing_aspects(&self) -> Vec<AgentId> {
        self.missing_aspects.keys().cloned().collect()
    }

    /// Returns true if the given agent is missing all of the given aspects for the given entry.
    /// That is: if all of the aspects are recorded as missing for that agent.
    /// If one of the given aspects is not in that vector of missing entries, the agent is supposed
    /// to have it and this function returns false.
    pub fn agent_is_missing_all_aspects(
        &self,
        agent_id: &AgentId,
        entry_hash: &EntryHash,
        aspects: &Vec<AspectHash>,
    ) -> bool {
        let maybe_agent_map = self.missing_aspects.get(agent_id);
        if maybe_agent_map.is_none() {
            return false;
        }
        let map_for_agent = maybe_agent_map.unwrap();

        let maybe_vec_of_missing_aspects_for_entry = map_for_agent.get(entry_hash);
        if maybe_vec_of_missing_aspects_for_entry.is_none() {
            return false;
        }

        let missing_aspects_for_entry = maybe_vec_of_missing_aspects_for_entry.unwrap();

        // We check that every of the given aspects is the missing list.
        // If one is missing from the missing list this block returns some
        // and the whole function returns false.
        for aspect in aspects {
            if !missing_aspects_for_entry.contains(aspect) {
                return false;
            }
        }

        true
    }

    pub fn agent_is_missing_some_aspect_for_entry(
        &self,
        agent_id: &AgentId,
        entry_hash: &EntryHash,
    ) -> bool {
        let maybe_agent_map = self.missing_aspects.get(agent_id);
        if maybe_agent_map.is_none() {
            return false;
        }
        maybe_agent_map.unwrap().get(entry_hash).is_some()
    }

    pub fn join_agent(&mut self, agent_id: AgentId, uri: Lib3hUri) -> Sim2hResult<()> {
        let location = calc_location_for_id(&self.crypto, &agent_id.to_string())?;
        self.agents.insert(agent_id, AgentInfo { uri, location });
        Ok(())
    }

    pub fn remove_agent(&mut self, agent_id: &AgentId) -> usize {
        self.agents.remove(agent_id);
        self.missing_aspects.remove(agent_id);
        self.agents.len()
    }

    pub fn agent_id_to_uri(&self, agent_id: &AgentId) -> Option<Lib3hUri> {
        for (found_agent, info) in self.agents.iter() {
            if found_agent == agent_id {
                return Some(info.uri.clone());
            }
        }
        None
    }

    pub(crate) fn all_agents(&self) -> &HashMap<AgentId, AgentInfo> {
        &self.agents
    }

    pub(crate) fn agents_supposed_to_hold_entry(
        &self,
        entry_location: Location,
        redundant_count: u64,
    ) -> HashMap<AgentId, AgentInfo> {
        self.agents
            .iter()
            .filter(|(_agent, info)| {
                naive_sharding_should_store(
                    info.location,
                    entry_location,
                    self.agents.len() as u64,
                    redundant_count,
                )
            })
            .map(|(e, v)| (e.clone(), v.clone()))
            .collect()
    }

    pub fn all_aspects(&self) -> &AspectList {
        &self.all_aspects_hashes
    }

    pub fn aspects_in_shard_for_agent(&self, agent: &AgentId, redundant_count: u64) -> AspectList {
        let agent_loc = self
            .agents
            .get(agent)
            .expect("cannot fetch aspects for unknown agent")
            .location;
        self.all_aspects_hashes
            .filtered_by_entry_hash(|entry_hash| {
                naive_sharding_should_store(
                    agent_loc,
                    entry_location(&self.crypto, &entry_hash),
                    self.agents.len() as u64,
                    redundant_count,
                )
            })
    }

    pub fn add_aspect(&mut self, entry_address: EntryHash, aspect_address: AspectHash) {
        self.all_aspects_hashes.add(entry_address, aspect_address);
    }
}

// TODO: unify with AspectMap
#[derive(Clone, Debug)]
pub struct AspectList(HashMap<EntryHash, Vec<AspectHash>>);
impl AspectList {
    /// Returns an AspectList list that contains every entry aspect
    /// in self that is not in other.
    pub fn diff(&self, other: &AspectList) -> AspectList {
        let self_set = HashSet::<(EntryHash, AspectHash)>::from(self);
        let other_set = HashSet::<(EntryHash, AspectHash)>::from(other);
        AspectList::from(&self_set.difference(other_set))
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

    pub fn aspect_hashes(&self) -> Vec<AspectHash> {
        let mut result = Vec::new();
        for (_, aspects) in self.0.iter() {
            result.append(&mut aspects.clone());
        }
        result
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

    pub fn filtered_by_entry_hash<F: FnMut(&EntryHash) -> bool>(
        &self,
        mut filter_fn: F,
    ) -> AspectList {
        AspectList::from(
            self.0
                .iter()
                .filter(|(entry_hash, _)| filter_fn(entry_hash))
                .map(|(e, v)| (e.clone(), v.clone()))
                .collect::<HashMap<EntryHash, Vec<AspectHash>>>(),
        )
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AgentId;
    use lib3h_protocol::uri::Lib3hUri;
    use lib3h_sodium::SodiumCryptoSystem;
    use std::convert::TryFrom;

    #[test]
    fn space_can_add_and_remove_agents() {
        let mut space = Space::new(Box::new(SodiumCryptoSystem::new()));
        let agent =
            AgentId::from("HcSCJCqoIY3uwiw34acyvNmJMyzkk4y9groHdYKBekqp7y48mvwfVTQQkzcjnfz");
        assert_eq!(space.agents.len(), 0);
        space
            .join_agent(
                agent.clone(),
                Lib3hUri::try_from("ws://someagenturi.com:9000").unwrap(),
            )
            .expect("should work");
        assert_eq!(space.agents.len(), 1);
        let entry_hash_1 = EntryHash::from("entry_hash_1");
        let aspect_hash_1 = AspectHash::from("aspect_hash_1");
        space.add_missing_aspect(agent.clone(), entry_hash_1.clone(), aspect_hash_1.clone());
        assert_eq!(space.agents_with_missing_aspects(), vec![agent.clone()]);

        assert_eq!(space.remove_agent(&agent), 0);
        assert_eq!(space.agents.len(), 0);
        // when removing the agent it's data in the missing_aspects list should also be cleared
        assert_eq!(space.agents_with_missing_aspects(), vec![]);
    }

    #[test]
    fn space_can_add_and_remove_missing_aspects() {
        let mut space = Space::new(Box::new(SodiumCryptoSystem::new()));
        let agent = AgentId::from("test-agent");

        assert!(space.agents_with_missing_aspects().is_empty());

        // Adding and removing one aspect and checking if agents_with_missing_aspects()
        // returns correct agent list:
        let entry_hash_1 = EntryHash::from("entry_hash_1");
        let aspect_hash_1 = AspectHash::from("aspect_hash_1");

        space.add_missing_aspect(agent.clone(), entry_hash_1.clone(), aspect_hash_1.clone());
        assert_eq!(space.agents_with_missing_aspects(), vec![agent.clone()]);
        space.remove_missing_aspect(&agent, &entry_hash_1, &aspect_hash_1);
        assert!(space.agents_with_missing_aspects().is_empty());

        // Adding two aspects, removing one first and then the other one and checking if
        // agents_with_missing_aspects returns correct agent lists.
        let aspect_hash_2 = AspectHash::from("aspect_hash_2");

        space.add_missing_aspect(agent.clone(), entry_hash_1.clone(), aspect_hash_1.clone());
        space.add_missing_aspect(agent.clone(), entry_hash_1.clone(), aspect_hash_2.clone());
        assert_eq!(space.agents_with_missing_aspects(), vec![agent.clone()]);
        space.remove_missing_aspect(&agent, &entry_hash_1, &aspect_hash_1);
        assert_eq!(space.agents_with_missing_aspects(), vec![agent.clone()]);
        space.remove_missing_aspect(&agent, &entry_hash_1, &aspect_hash_2);
        assert!(space.agents_with_missing_aspects().is_empty());

        // Adding two aspects of different entries, removing one first and then the other one
        // and checking if agents_with_missing_aspects returns correct agent lists.
        let entry_hash_2 = EntryHash::from("entry_hash_2");

        space.add_missing_aspect(agent.clone(), entry_hash_1.clone(), aspect_hash_1.clone());
        space.add_missing_aspect(agent.clone(), entry_hash_2.clone(), aspect_hash_2.clone());
        assert_eq!(space.agents_with_missing_aspects(), vec![agent.clone()]);
        space.remove_missing_aspect(&agent, &entry_hash_2, &aspect_hash_2);
        assert_eq!(space.agents_with_missing_aspects(), vec![agent.clone()]);
        space.remove_missing_aspect(&agent, &entry_hash_1, &aspect_hash_1);
        assert!(space.agents_with_missing_aspects().is_empty());
    }

    #[test]
    fn space_can_tell_if_agent_is_missing_all_aspects() {
        let mut space = Space::new(Box::new(SodiumCryptoSystem::new()));
        let agent = AgentId::from("test-agent");
        let entry_hash_1 = EntryHash::from("entry_hash_1");
        let entry_hash_2 = EntryHash::from("entry_hash_2");
        let aspect_hash_1_1 = AspectHash::from("aspect_hash_1_1");
        let aspect_hash_1_2 = AspectHash::from("aspect_hash_1_2");
        let aspect_hash_2_1 = AspectHash::from("aspect_hash_2_1");
        //let aspect_hash_2_2 = AspectHash::from("aspect_hash_2_2");
        //let aspect_hash_2_3 = AspectHash::from("aspect_hash_2_3");

        assert!(!space.agent_is_missing_all_aspects(
            &agent,
            &entry_hash_1,
            &vec![aspect_hash_1_1.clone()]
        ));
        space.add_missing_aspect(agent.clone(), entry_hash_1.clone(), aspect_hash_1_1.clone());
        assert!(space.agent_is_missing_all_aspects(
            &agent,
            &entry_hash_1,
            &vec![aspect_hash_1_1.clone()]
        ));
        space.add_missing_aspect(agent.clone(), entry_hash_2.clone(), aspect_hash_2_1.clone());
        assert!(space.agent_is_missing_all_aspects(
            &agent,
            &entry_hash_1,
            &vec![aspect_hash_1_1.clone()]
        ));

        assert!(!space.agent_is_missing_all_aspects(
            &agent,
            &entry_hash_1,
            &vec![aspect_hash_1_1.clone(), aspect_hash_1_2.clone()]
        ));
        space.add_missing_aspect(agent.clone(), entry_hash_1.clone(), aspect_hash_1_2.clone());
        assert!(space.agent_is_missing_all_aspects(
            &agent,
            &entry_hash_1,
            &vec![aspect_hash_1_1.clone(), aspect_hash_1_2.clone()]
        ));
    }
}
