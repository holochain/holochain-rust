use lib3h_protocol::types::{AspectHash, EntryHash};
use std::collections::{HashMap, HashSet};

pub type AspectSet = HashSet<AspectHash>;

#[derive(Debug)]
pub struct AspectMap(HashMap<EntryHash, AspectSet>);
impl AspectMap {
    /// Returns an AspectMap list that contains every entry aspect
    /// in self that is not in other.
    pub fn diff(&self, other: &AspectMap) -> AspectMap {
        let self_set = HashSet::<(EntryHash, AspectHash)>::from(self);
        let other_set = HashSet::<(EntryHash, AspectHash)>::from(other);
        AspectMap::from(
            &self_set
                .difference(&other_set)
                .cloned()
                .collect::<HashSet<(EntryHash, AspectHash)>>(),
        )
    }

    pub fn add(&mut self, entry_address: EntryHash, aspect_address: AspectHash) {
        self.0
            .entry(entry_address)
            .or_insert_with(HashSet::new)
            .insert(aspect_address);
    }

    pub fn entry_addresses(&self) -> impl Iterator<Item = &EntryHash> {
        self.0.keys()
    }

    pub fn per_entry(&self, entry_address: &EntryHash) -> Option<&HashSet<AspectHash>> {
        self.0.get(entry_address)
    }

    pub fn aspect_hashes(&self) -> HashSet<&AspectHash> {
        self.0.values().flat_map(|v| v.into_iter()).collect()
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

impl From<HashMap<EntryHash, HashSet<AspectHash>>> for AspectMap {
    fn from(map: HashMap<EntryHash, HashSet<AspectHash>>) -> AspectMap {
        AspectMap { 0: map }
    }
}

impl From<&AspectMap> for HashSet<(EntryHash, AspectHash)> {
    fn from(a: &AspectMap) -> HashSet<(EntryHash, AspectHash)> {
        let mut result = HashSet::new();
        for (entry_address, aspect_list) in a.0.iter() {
            for aspect_address in aspect_list {
                result.insert((entry_address.clone(), aspect_address.clone()));
            }
        }
        result
    }
}

// TODO: is this needed?
impl From<&HashSet<(EntryHash, AspectHash)>> for AspectMap {
    fn from(s: &HashSet<(EntryHash, AspectHash)>) -> AspectMap {
        let mut result: HashMap<EntryHash, HashSet<AspectHash>> = HashMap::new();
        for (entry_address, aspect_address) in s {
            if !result.contains_key(entry_address) {
                result.insert(entry_address.clone(), HashSet::new());
            }
            result
                .get_mut(entry_address)
                .unwrap()
                .insert(aspect_address.clone());
        }
        AspectMap::from(result)
    }
}
