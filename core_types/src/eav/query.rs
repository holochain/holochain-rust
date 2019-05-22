use eav::eavi::{Attribute, Entity, EntityAttributeValueIndex, Value};
use std::collections::BTreeSet;

/// Represents a set of filtering operations on the EAVI store.
pub struct EaviQuery<'a> {
    pub entity: EntityFilter<'a>,
    pub attribute: AttributeFilter<'a>,
    pub value: ValueFilter<'a>,
    pub tombstone: Option<AttributeFilter<'a>>,
    pub index: IndexFilter,
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
            None,
        )
    }
}

impl<'a> EaviQuery<'a> {
    pub fn new(
        entity: EntityFilter<'a>,
        attribute: AttributeFilter<'a>,
        value: ValueFilter<'a>,
        index: IndexFilter,
        tombstone: Option<AttributeFilter<'a>>,
    ) -> Self {
        Self {
            entity,
            attribute,
            value,
            tombstone,
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
                .filter_map(|eavi| {
                    let reduced_value = iter2.clone().fold((None, false), |eavi_option, eavi_fold| {
                        if eavi_option.1 {
                            eavi_option
                        } else {
                            let fold_query = EaviQuery::new(
                                Some(eavi.entity()).into(),
                                Some(eavi.attribute()).into(),
                                Some(eavi.value()).into(),
                                IndexFilter::LatestByAttribute,
                                None,
                            );
                            if EaviQuery::eav_check(
                                &eavi_fold,
                                &fold_query.entity,
                                &self.attribute,
                                &fold_query.value,
                            ) {
                                if *&self
                                    .tombstone()
                                    .as_ref()
                                    .map(|s| s.check(eavi_fold.attribute()))
                                    .unwrap_or(true)
                                    .clone()
                                {
                                    (
                                        Some(eavi_fold),
                                        *&self
                                            .tombstone()
                                            .as_ref()
                                            .map(|_| true)
                                            .unwrap_or(false)
                                            .clone(),
                                    )
                                } else {
                                    (Some(eavi_fold), false)
                                }
                            } else {
                                eavi_option
                            }
                        }
                    });
                    reduced_value.0
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
    pub fn tombstone(&self) -> &Option<AttributeFilter<'a>> {
        &self.tombstone
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
        val.map(EavFilter::single).unwrap_or_default()
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
