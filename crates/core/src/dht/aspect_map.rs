use crate::holochain_wasm_utils::holochain_persistence_api::cas::content::AddressableContent;
use holochain_core_types::network::entry_aspect::EntryAspect;
use lib3h_protocol::types::{AspectHash, EntryHash};
use std::collections::{HashMap, HashSet};

pub type AspectSet = HashSet<AspectHash>;

pub type AspectMapBare = HashMap<EntryHash, AspectSet>;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AspectMap(AspectMapBare);
impl AspectMap {
    pub fn new() -> Self {
        Self::default()
    }

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

    pub fn bare(&self) -> &AspectMapBare {
        &self.0
    }

    pub fn contains(&self, aspect: &EntryAspect) -> bool {
        let entry_address: EntryHash = aspect.entry_address().into();
        let entry_aspect_address = aspect.address();
        self.0
            .get(&entry_address)
            .map(|set| set.contains(&entry_aspect_address))
            .unwrap_or_default()
    }

    pub fn add(&mut self, aspect: &EntryAspect) {
        let entry_address = aspect.entry_address().into();
        let entry_aspect_address = aspect.address().into();

        self.0
            .entry(entry_address)
            .or_insert_with(HashSet::new)
            .insert(entry_aspect_address);
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

    pub fn merge(map1: &AspectMap, map2: &AspectMap) -> AspectMap {
        map1.0
            .keys()
            .chain(map2.0.keys())
            .map(|entry| {
                let merged = map1
                    .0
                    .get(entry)
                    .unwrap_or(&HashSet::new())
                    .union(map2.0.get(entry).unwrap_or(&HashSet::new()))
                    .cloned()
                    .collect();
                (entry.clone(), merged)
            })
            .collect::<AspectMapBare>()
            .into()
    }
}

impl From<AspectMapBare> for AspectMap {
    fn from(map: AspectMapBare) -> AspectMap {
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

pub type AspectVecMap = HashMap<EntryHash, Vec<AspectHash>>;
impl From<AspectMap> for AspectVecMap {
    fn from(map: AspectMap) -> AspectVecMap {
        let mut new_map = HashMap::new();
        map.0.into_iter().for_each(|(entry, set)| {
            let vec = set.into_iter().collect();
            new_map.insert(entry, vec);
        });
        new_map
    }
}

// TODO: is this needed?
impl From<&HashSet<(EntryHash, AspectHash)>> for AspectMap {
    fn from(s: &HashSet<(EntryHash, AspectHash)>) -> AspectMap {
        let mut result: AspectMapBare = HashMap::new();
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

#[cfg(test)]
mod tests {

    use super::*;
    use sim1h::aspect::fixture::content_aspect_fresh;

    #[test]
    fn test_address_map_add_and_contains() {
        let mut map = AspectMap::new();
        let test_aspect = content_aspect_fresh();
        assert_eq!(map.bare().len(), 0);
        map.add(&test_aspect);
        assert_eq!(map.bare().len(), 1);
        assert!(map.contains(&test_aspect));
        let other_test_aspect = content_aspect_fresh();
        assert!(!map.contains(&other_test_aspect));
    }

    #[test]
    fn test_merge_address_maps_merges_entries() {
        let mut map1: AspectMapBare = HashMap::new();
        let mut map2: AspectMapBare = HashMap::new();
        map1.insert("a".into(), vec!["x".into()].into_iter().collect());
        map2.insert("b".into(), vec!["y".into()].into_iter().collect());
        let (map1, map2) = (map1.into(), map2.into());
        let merged = AspectMap::merge(&map1, &map2);
        let merged2 = AspectMap::merge(&map2, &map1);
        assert_eq!(merged.0, merged2.0);
        assert_eq!(merged.0.len(), 2);
        assert_eq!(merged.0.get(&EntryHash::from("a")).unwrap().len(), 1);
        assert_eq!(merged.0.get(&EntryHash::from("b")).unwrap().len(), 1);
    }

    #[test]
    fn test_merge_address_maps_merges_aspects_1() {
        let mut map1: AspectMapBare = HashMap::new();
        let mut map2: AspectMapBare = HashMap::new();
        map1.insert("a".into(), vec!["x".into()].into_iter().collect());
        map2.insert(
            "a".into(),
            vec!["x".into(), "y".into()].into_iter().collect(),
        );
        let (map1, map2) = (map1.into(), map2.into());
        let merged = AspectMap::merge(&map1, &map2);
        let merged2 = AspectMap::merge(&map1, &map2);
        assert_eq!(merged.0, merged2.0);
        assert_eq!(merged.0.len(), 1);
        assert_eq!(merged.0.get(&EntryHash::from("a")).unwrap().len(), 2);
    }

    #[test]
    fn test_merge_address_maps_merges_aspects_2() {
        // Full merged outcome should be:
        // a => x, y, z
        // b => u, v, w
        let mut map1: AspectMapBare = HashMap::new();
        let mut map2: AspectMapBare = HashMap::new();
        map1.insert(
            "a".into(),
            vec!["x".into(), "y".into()].into_iter().collect(),
        );
        map1.insert(
            "b".into(),
            vec!["u".into(), "v".into()].into_iter().collect(),
        );

        map2.insert(
            "a".into(),
            vec!["y".into(), "z".into()].into_iter().collect(),
        );
        map2.insert(
            "b".into(),
            vec!["v".into(), "w".into()].into_iter().collect(),
        );
        let (map1, map2) = (map1.into(), map2.into());
        let merged = AspectMap::merge(&map1, &map2);
        let merged2 = AspectMap::merge(&map2, &map1);
        assert_eq!(merged.0, merged2.0);
        assert_eq!(merged.0.len(), 2);
        assert_eq!(merged.0.get(&EntryHash::from("a")).unwrap().len(), 3);
        assert_eq!(merged.0.get(&EntryHash::from("b")).unwrap().len(), 3);
    }

}
