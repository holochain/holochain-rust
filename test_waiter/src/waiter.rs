use crate::{
    network::{FullSyncNetworkModel, NetworkModel},
    types::{EffectConcrete, NodeId, Observation},
};

pub struct Waiter<N: NetworkModel> {
    network_model: N,
    pending_effects: Vec<EffectConcrete>,
}

impl Waiter<FullSyncNetworkModel> {
    pub fn new(nodes: Vec<NodeId>) -> Self {
        Self {
            network_model: FullSyncNetworkModel::new(nodes),
            pending_effects: Vec::new(),
        }
    }

    pub fn is_consistent(&self, node: &NodeId) -> bool {
        self.pending_effects
            .iter()
            .filter(|eff| eff.target_node == *node)
            .collect::<Vec<_>>()
            .is_empty()
    }

    pub fn is_consistent_all(&self) -> bool {
        self.pending_effects.is_empty()
    }

    pub fn reduce_observation(&mut self, o: &Observation) {
        let mut fx = self.network_model.determine_effects(o);
        // println!(
        //     "{:?} -> {:?}",
        //     o,
        //     fx.iter()
        //         .map(|eff| eff.description.clone())
        //         .collect::<Vec<_>>()
        // );
        self.pending_effects.append(&mut fx);
        self.pending_effects
            .retain(|eff| !eff.matches_observation(o));
    }
}

pub type FullSyncWaiter = Waiter<FullSyncNetworkModel>;

#[cfg(test)]
mod tests {
    use super::*;
    use holochain_core::{action::Action::*, network::entry_with_header::EntryWithHeader};
    use holochain_core_types::{
        cas::content::AddressableContent, chain_header::test_chain_header, entry::Entry,
        json::JsonString,
    };
    use std::panic;

    fn mk_entry(ty: &'static str, content: &'static str) -> Entry {
        Entry::App(ty.into(), JsonString::from_json(content))
    }

    fn mk_entry_wh(entry: Entry) -> EntryWithHeader {
        EntryWithHeader {
            entry,
            header: test_chain_header(),
        }
    }

    fn test_nodes() -> Vec<NodeId> {
        ["alise", "bobo", "lola"]
            .into_iter()
            .map(|n| n.to_string())
            .collect()
    }

    fn test_waiter() -> FullSyncWaiter {
        Waiter::new(test_nodes())
    }

    #[test]
    fn scenario_publish_and_hold() {
        let mut waiter = test_waiter();
        let entry = mk_entry("t1", "x");
        let entry_wh = mk_entry_wh(entry.clone());
        let commit_key = (entry.clone(), None, Vec::new());

        waiter.reduce_observation(&Observation {
            node: "alise".into(),
            action: Commit(commit_key),
        });
        assert_eq!(waiter.pending_effects.len(), 0);

        waiter.reduce_observation(&Observation {
            node: "alise".into(),
            action: Publish(entry.address()),
        });
        assert_eq!(waiter.pending_effects.len(), 3);

        let hold = Hold(entry_wh.clone());

        waiter.reduce_observation(&Observation {
            node: "alise".into(),
            action: hold.clone(),
        });
        assert_eq!(waiter.pending_effects.len(), 2);

        waiter.reduce_observation(&Observation {
            node: "bobo".into(),
            action: hold.clone(),
        });
        assert_eq!(waiter.pending_effects.len(), 1);
        assert!(waiter.is_consistent(&"bobo".to_string()));

        waiter.reduce_observation(&Observation {
            node: "lola".into(),
            action: hold.clone(),
        });
        assert_eq!(waiter.pending_effects.len(), 0);

        assert!(waiter.is_consistent_all());
    }
}
