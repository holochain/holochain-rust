use eav::eavi::{Attribute, Entity, EntityAttributeValueIndex, Value};
use std::collections::BTreeSet;

/// Represents a set of filtering operations on the EAVI store.
pub struct EaviQuery<'a> {
    entity: EntityFilter<'a>,
    attribute: AttributeFilter<'a>,
    value: ValueFilter<'a>,
    index: IndexFilter,
}

type EntityFilter<'a> = EavFilter<'a, Entity>;
type AttributeFilter<'a> = EavFilter<'a, Attribute>;
type ValueFilter<'a> = EavFilter<'a, Value>;

impl<'a> Default for EaviQuery<'a> {
    fn default() -> EaviQuery<'a> {
        EaviQuery::new(
            Default::default(),
            Default::default(),
            Default::default(),
            IndexFilter::LatestByAttribute,
        )
    }
}

impl<'a> EaviQuery<'a> {
    pub fn new(
        entity: EntityFilter<'a>,
        attribute: AttributeFilter<'a>,
        value: ValueFilter<'a>,
        index: IndexFilter,
    ) -> Self {
        Self {
            entity,
            attribute,
            value,
            index,
        }
    }

    pub fn run<I>(&self, iter: I) -> BTreeSet<EntityAttributeValueIndex>
    where
        I: Clone + Iterator<Item = EntityAttributeValueIndex> + 'a,
    {
        let iter2 = iter.clone();
        let filtered = iter
            .clone()
            .filter(|eavi| EaviQuery::eav_check(&eavi, &self.entity, &self.attribute, &self.value));

        match self.index {
            IndexFilter::LatestByAttribute => filtered
                .filter(|eavi| {
                    iter2
                        .clone()
                        .filter(|eavi_inner| {
                            EaviQuery::eav_check(
                                &eavi_inner,
                                &Some(eavi.entity()).into(),
                                &self.attribute,
                                &Some(eavi.value()).into(),
                            )
                        })
                        .last()
                        .map(|latest| latest.index() == eavi.index())
                        .unwrap_or(false)
                })
                .collect(),
            IndexFilter::Range(start, end) => filtered
                .filter(|eavi| {
                    start.map(|lo| lo <= eavi.index()).unwrap_or(true)
                        && end.map(|hi| eavi.index() <= hi).unwrap_or(true)
                })
                .collect(),
        }
    }

    fn eav_check(
        eavi: &EntityAttributeValueIndex,
        e: &EntityFilter<'a>,
        a: &AttributeFilter<'a>,
        v: &ValueFilter<'a>,
    ) -> bool {
        e.check(eavi.entity()) && a.check(eavi.attribute()) && v.check(eavi.value())
    }

    pub fn entity(&self) -> &EntityFilter<'a> {
        &self.entity
    }
    pub fn attribute(&self) -> &AttributeFilter<'a> {
        &self.attribute
    }
    pub fn value(&self) -> &ValueFilter<'a> {
        &self.value
    }
    pub fn index(&self) -> &IndexFilter {
        &self.index
    }
}

pub struct EavFilter<'a, T: 'a + Eq>(Box<dyn Fn(T) -> bool + 'a>);

impl<'a, T: 'a + Eq> EavFilter<'a, T> {
    pub fn single(val: T) -> Self {
        Self(Box::new(move |v| v == val))
    }

    pub fn multiple(vals: Vec<T>) -> Self {
        Self(Box::new(move |val| vals.iter().any(|v| *v == val)))
    }

    pub fn predicate<F>(predicate: F) -> Self
    where
        F: Fn(T) -> bool + 'a,
    {
        Self(Box::new(predicate))
    }

    pub fn check(&self, val: T) -> bool {
        self.0(val)
    }
}

impl<'a, T: Eq> Default for EavFilter<'a, T> {
    fn default() -> EavFilter<'a, T> {
        Self(Box::new(|_| true))
    }
}

impl<'a, T: Eq> From<Option<T>> for EavFilter<'a, T> {
    fn from(val: Option<T>) -> EavFilter<'a, T> {
        val.map(|v| EavFilter::single(v)).unwrap_or_default()
    }
}

impl<'a, T: Eq> From<Vec<T>> for EavFilter<'a, T> {
    fn from(vals: Vec<T>) -> EavFilter<'a, T> {
        EavFilter::multiple(vals)
    }
}

/// Specifies options for filtering on Index:
/// Range returns all results within a particular range of indices.
/// LatestByAttribute is more complex. It first does a normal filter by E, A, and V.
/// Then, for each group of items which differ *only* by Attribute and Index, only the item with
/// highest Index is retained for that grouping.
#[derive(Clone, Debug)]
pub enum IndexFilter {
    LatestByAttribute,
    Range(Option<i64>, Option<i64>),
}
