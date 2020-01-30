use crate::*;
use std::{cell::RefCell, hash::Hash};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MonoRef<T>(std::sync::Arc<T>);

impl<T> MonoRef<T> {
    pub fn new(t: T) -> Self {
        Self(std::sync::Arc::new(t))
    }
}

impl<T> std::ops::Deref for MonoRef<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::convert::AsRef<T> for MonoRef<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> std::borrow::Borrow<T> for MonoRef<T> {
    fn borrow(&self) -> &T {
        &self.0
    }
}

impl MonoRef<String> {
    pub fn as_agent_id(&self) -> AgentId {
        self.as_ref().clone().into()
    }

    pub fn as_space_hash(&self) -> SpaceHash {
        self.as_ref().clone().into()
    }

    pub fn as_entry_hash(&self) -> EntryHash {
        self.as_ref().clone().into()
    }

    pub fn as_aspect_hash(&self) -> AspectHash {
        self.as_ref().clone().into()
    }
}

impl From<AgentId> for MonoRef<String> {
    fn from(a: AgentId) -> Self {
        Self::new(a.into())
    }
}

impl From<SpaceHash> for MonoRef<String> {
    fn from(s: SpaceHash) -> Self {
        Self::new(s.into())
    }
}

impl From<EntryHash> for MonoRef<String> {
    fn from(e: EntryHash) -> Self {
        Self::new(e.into())
    }
}

impl From<AspectHash> for MonoRef<String> {
    fn from(a: AspectHash) -> Self {
        Self::new(a.into())
    }
}

#[derive(Clone)]
pub struct MonoRefCache<T: Clone + Eq + Hash>(RefCell<im::HashMap<MonoRef<T>, MonoRef<T>>>);

impl<T: Clone + Eq + Hash> MonoRefCache<T> {
    pub fn new() -> Self {
        Self(RefCell::new(im::HashMap::new()))
    }

    pub fn get(&self, t: T) -> MonoRef<T> {
        let m = MonoRef::new(t);
        self.0.borrow_mut().entry(m.clone()).or_insert(m).clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ref_returns_same_address() {
        let i1 = MonoRef::new(42_usize);
        let i2 = i1.clone();
        let i3 = MonoRef::new(42_usize);

        let r1: &usize = &i1;
        let r2: &usize = &i2;
        let r3: &usize = &i3;

        assert_eq!(&format!("{:p}", r1), &format!("{:p}", r2));
        assert_ne!(&format!("{:p}", r1), &format!("{:p}", r3));
    }

    #[test]
    fn cache_returns_same_address() {
        let c = <MonoRefCache<usize>>::new();

        let i1 = c.get(42_usize);
        let i2 = c.get(42_usize);
        let i3 = MonoRef::new(42_usize);

        let r1: &usize = &i1;
        let r2: &usize = &i2;
        let r3: &usize = &i3;

        assert_eq!(&format!("{:p}", r1), &format!("{:p}", r2));
        assert_ne!(&format!("{:p}", r1), &format!("{:p}", r3));
    }
}
