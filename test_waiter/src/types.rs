use holochain_core::action::Action;
use std::rc::Rc;

pub type NodeId = String;

#[derive(Clone, Debug)]
pub struct Observation {
    pub action: Action,
    pub node: NodeId,
}

#[derive(Clone)]
pub struct EffectAbstract {
    pub description: String,
    pub predicate: Rc<Box<Fn(&Action) -> bool>>,
    pub group: EffectGroup,
}

#[derive(Clone)]
pub struct EffectConcrete {
    pub description: String,
    pub predicate: Rc<Box<Fn(&Action) -> bool>>,
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
