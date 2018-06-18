use error::HolochainError;

/// trait that defines the persistence functionality that hc_core requires
pub trait Persister {
    fn save(&self);
    fn load(&self) -> Result<(), HolochainError>;
}

pub struct SimplePersister {}

impl Persister for SimplePersister {
    fn save(&self) {}
    fn load(&self) -> Result<(), HolochainError> {
        Ok(())
    }
}
