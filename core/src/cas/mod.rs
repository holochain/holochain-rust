use hash::HashString;
use error::HolochainError;

type Address = HashString;
type Content = String;

pub trait AddressableContent {
    fn address(&self) -> Address;
    fn content(&self) -> Content;
}

pub trait Store {
    fn store(&self, content: &AddressableContent) -> Result<(), HolochainError>;
    fn contains(&self, address: &Address) -> Result<bool, HolochainError>;
    fn retrieve(&self, address: &Address) -> Result<Content, HolochainError>;
}
