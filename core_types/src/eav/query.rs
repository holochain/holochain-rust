use eav::eavi::{Attribute, Entity, EntityAttributeValueIndex, Value};
use std::collections::BTreeSet;

/// Represents a set of filtering operations on the EAVI store.
pub struct EaviQuery<'a> {
    ///represents a filter for the Entity
    pub entity: EntityFilter<'a>,
    ///represents a filter for the Attribute
    pub attribute: AttributeFilter<'a>,
    ///represents a filter for the Value
    pub value: ValueFilter<'a>,
    ///For this query system we are able to provide a tombstone set on the query level which allows us to specify which Attribute match should take precedent over the others.
    ///This is useful for some of the Link CRDT operations we are doing. Note that if no tombstone is found, the latest entry is returned
    ///from the subset that is obtained. The tombstone is optional so if it is not supplied, no tombstone check will be done.
    ///Currently the Tombstone does not work on an IndexByRange IndexFilter and will operate as if the tombstone was not set
    pub tombstone: Option<AttributeFilter<'a>>,
    ///represents a filter for the Index
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

///Creates a query for the EAVI system
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

    ///This runs the query based the query configuration we have given
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
                    // this fold reduces a set of matched (e,a,v) values but makes sure the tombstone value takes priority.
                    //the starting point is a tuple (Option,bool) which translates to a cnodition in which we have found our tombstone value and it also matches our tombstone
                    let reduced_value =
                        iter2.clone().fold((None, false), |eavi_option, eavi_fold| {
                            //if tombstone match is found, do not check the rest
                            if eavi_option.1 {
                                eavi_option
                            } else {
                                //create eavi query without tombstone set, they are two levels here. we have to make sure that the values match our eav set but later on we check if that value given also matches our tombstone condition
                                let fold_query = EaviQuery::new(
                                    Some(eavi.entity()).into(),
                                    Some(eavi.attribute()).into(),
                                    Some(eavi.value()).into(),
                                    IndexFilter::LatestByAttribute,
                                    None,
                                );
                                //check if the value matches our initial condition
                                if EaviQuery::eav_check(
                                    &eavi_fold,
                                    &fold_query.entity,
                                    &self.attribute,
                                    &fold_query.value,
                                ) {
                                    //check if tombstone condition is met
                                    if *&self
                                        .tombstone()
                                        .as_ref()
                                        .map(|s| s.check(eavi_fold.attribute()))
                                        .unwrap_or(true)
                                        .clone()
                                    {
                                        //if attrribute is found return the value plus the tombstone boolean set to true
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
                                        //return value that signifies value has been found but tombstone has not been found
                                        (Some(eavi_fold), false)
                                    }
                                } else {
                                    //if set does not match, just return last value of eavi_option
                                    eavi_option
                                }
                            }
                        });

                    //at the end just return initial value of tombstone
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

/// Represents a fitler type which takes in a function to match on
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
