use crate::*;

/// ref counted - lets us override some debugging
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MonoRef<T>(std::sync::Arc<T>);

impl std::fmt::Debug for MonoRef<AgentId> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AgentId({})", self.0)
    }
}
impl std::fmt::Debug for MonoRef<SpaceHash> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SpaceHash({})", self.0)
    }
}
impl std::fmt::Debug for MonoRef<EntryHash> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EntryHash({})", self.0)
    }
}
impl std::fmt::Debug for MonoRef<AspectHash> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AspectHash({})", self.0)
    }
}
impl std::fmt::Debug for MonoRef<Lib3hUri> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Uri({})", self.0)
    }
}

/*
// WTF - why can't we make a generic one too?
impl<T: std::fmt::Debug> std::fmt::Debug for MonoRef<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
*/

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

impl<T> From<T> for MonoRef<T> {
    fn from(t: T) -> MonoRef<T> {
        MonoRef::new(t)
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
    fn im_map_get_key_value_works() {
        let i1 = MonoRef::new(42_usize);

        let mut map = im::HashMap::new();
        map.insert(i1.clone(), (i1.clone(), 1));

        let i3 = MonoRef::new(42_usize);

        let r1: &usize = &i1;
        let (k, _v) = map.get(&42).unwrap();
        let r2: &usize = &k;

        let r3: &usize = &i3;

        assert_eq!(&format!("{:p}", r1), &format!("{:p}", r2));
        assert_ne!(&format!("{:p}", r1), &format!("{:p}", r3));
    }
}
