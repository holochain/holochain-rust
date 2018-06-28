use chain::pair::Pair;
use serde::Serialize;
use serde::Deserialize;

pub trait SourceChain<'de>: IntoIterator + Serialize + Deserialize<'de> {

    /// append a pair to the source chain if the pair and new chain are both valid, else panic
    fn push(&mut self, &Pair);

    /// returns an iterator referencing pairs from top (most recent) to bottom (genesis)
    fn iter(&self) -> std::slice::Iter<Pair>;

    /// returns true if system and dApp validation is successful
    fn validate(&self) -> bool;

    /// returns a pair for a given header hash
    fn get(&self, k: u64) -> Option<Pair>;

    /// returns a pair for a given entry hash
    fn get_entry(&self, k:u64) -> Option<Pair>;

}

#[cfg(test)]
mod tests {
    use super::Pair;
    use common::entry::Entry;
    use common::entry::Header;

    #[test]
    fn validate() {
        let e1 = Entry::new(&String::from("bar"));
        let h1 = Header::new(None, &e1);
        let p1 = Pair::new(&h1, &e1);

        assert!(p1.validate());
    }

    #[test]
    #[should_panic(expected = "attempted to create an invalid pair")]
    fn invalidate() {
        let e1 = Entry::new(&String::from("foo"));
        let e2 = Entry::new(&String::from("bar"));
        let h1 = Header::new(None, &e1);

        // header/entry mismatch, must panic!
        Pair::new(&h1, &e2);
    }
}
