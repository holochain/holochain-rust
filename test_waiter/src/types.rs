use holochain_core::action::Action;
use std::sync::Arc;

pub type NodeId = String;

#[derive(Clone, Debug)]
pub struct Observation {
    pub action: Action,
    pub node: NodeId,
}

#[derive(Clone)]
pub struct EffectAbstract {
    pub description: String,
    pub predicate: Arc<Box<Fn(&Action) -> bool + Send + Sync>>,
    pub group: EffectGroup,
}

#[derive(Clone)]
pub struct EffectConcrete {
    pub description: String,
    pub predicate: Arc<Box<Fn(&Action) -> bool + Send + Sync>>,
    pub source_node: NodeId,
    pub target_node: NodeId,
}

impl EffectConcrete {
    pub fn matches_observation(&self, o: &Observation) -> bool {
        (self.predicate)(&o.action) && self.target_node == o.node
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EffectGroup {
    Owner,
    Validators,
}
