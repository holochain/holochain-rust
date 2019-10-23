use eav::eavi::Attribute;

/// Represents a set of filtering operations on the EAVI store.
pub type EaviQuery<'a> = holochain_persistence_api::eav::query::EaviQuery<'a, Attribute>;
pub type EntityFilter<'a> = holochain_persistence_api::eav::query::EavFilter<'a, Attribute>;
pub type AttributeFilter<'a> = holochain_persistence_api::eav::query::EavFilter<'a, Attribute>;
pub type ValueFilter<'a> = holochain_persistence_api::eav::query::EavFilter<'a, Attribute>;
