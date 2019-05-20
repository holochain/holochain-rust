use crate::{
    causality::CausalityModel,
    types::{EffectAbstract, EffectConcrete, EffectGroup, NodeId, Observation},
};
use holochain_core::action::Action;

pub trait NetworkModel {
    fn determine_effects(&mut self, o: &Observation) -> Vec<EffectConcrete>;
}

pub struct FullSyncNetworkModel {
    agents: Vec<NodeId>,
    causality: CausalityModel,
}

impl NetworkModel for FullSyncNetworkModel {
    /// Run an Observation through both the CausalityModel and this NetworkModel
    /// to produce a collection of EffectConcrete to be used by the Waiter
    fn determine_effects(&mut self, o: &Observation) -> Vec<EffectConcrete> {
        self.causality
            .resolve_action(&o.action)
            .into_iter()
            .flat_map(|eff| self.concretize_effect(o, eff))
            .collect()
    }
}

impl FullSyncNetworkModel {
    pub fn new(agents: Vec<NodeId>) -> Self {
        Self {
            agents,
            causality: CausalityModel::new(),
        }
    }

    /// Take an EffectAbstract, along with the Observation that produced it,
    /// and generate an iterator of EffectConcrete (mainly based on the EffectGroup)
    fn concretize_effect<'a>(
        &self,
        obs: &'a Observation,
        eff: EffectAbstract,
    ) -> impl Iterator<Item = EffectConcrete> + 'a {
        let Observation { action, node } = obs;
        let EffectAbstract {
            description,
            predicate,
            group,
        } = eff;

        let target_nodes = match group {
            EffectGroup::Owner => vec![node.clone()],
            EffectGroup::Validators => self.validators(&action),
        };

        target_nodes
            .into_iter()
            .map(move |target_node| EffectConcrete {
                description: description.clone(),
                predicate: predicate.clone(),
                source_node: node.clone(),
                target_node: target_node,
            })
    }

    fn validators(&self, _action: &Action) -> Vec<NodeId> {
        // self.validator_resolver.resolve(action)
        self.agents.clone()
    }
}
