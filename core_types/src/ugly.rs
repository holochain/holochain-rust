//! Types to better represent ugly aspects of the code.
//! The idea is to bring more visibility to brittle or leaky abstractions, and to
//! make bad code less convenient to use, so we are more motivated to fix it.
//! Anything which is implemented using one of these types should be refactored
//! so that we no longer have to use that type.

/// Represents a value which may not have been initialized.
/// Isomorphic to `Option`, but semantically different.
pub enum Initable<T> {
    Init(T),
    Uninit,
}

impl<T> Initable<T> {
    pub fn to_option(val: Initable<T>) -> Option<T> {
        match val {
            Initable::Init(v) => Some(v),
            Initable::Uninit => None,
        }
    }

    pub fn expect(self, msg: &'static str) -> T {
        match self {
            Initable::Init(v) => v,
            Initable::Uninit => panic!(msg),
        }
    }
}

impl<T> From<Option<T>> for Initable<T> {
    fn from(val: Option<T>) -> Initable<T> {
        val.map(Initable::Init).unwrap_or(Initable::Uninit)
    }
}
