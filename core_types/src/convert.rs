/// An attempted conversion that consumes `self`, which may or may not be
/// expensive.
///
/// Library authors should not directly implement this trait, but should prefer
/// implementing the [`TryFrom`] trait, which offers greater flexibility and
/// provides an equivalent `TryInto` implementation for free, thanks to a
/// blanket implementation in the standard library. For more information on this,
/// see the documentation for [`Into`].
///
/// [`TryFrom`]: trait.TryFrom.html
/// [`Into`]: trait.Into.html
pub trait TryInto<T>: Sized {
    /// The type returned in the event of a conversion error.
    type Error;

    /// Performs the conversion.
    fn try_into(self) -> Result<T, Self::Error>;
}

/// Attempt to construct `Self` via a conversion.
pub trait TryFrom<T>: Sized {
    /// The type returned in the event of a conversion error.
    type Error;

    /// Performs the conversion.
    fn try_from(value: T) -> Result<Self, Self::Error>;
}

// TryFrom implies TryInto
impl<T, U> TryInto<U> for T where U: TryFrom<T>
{
    type Error = U::Error;

    fn try_into(self) -> Result<U, U::Error> {
        U::try_from(self)
    }
}

pub enum Infallible {}

// Infallible conversions are semantically equivalent to fallible conversions
// with an uninhabited error type.
impl<T, U> TryFrom<U> for T where T: From<U> {
    type Error = Infallible;

    fn try_from(value: U) -> Result<Self, Self::Error> {
        Ok(T::from(value))
    }
}
