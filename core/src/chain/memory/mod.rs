use std;

use chain::{entry::Entry, pair::Pair, SourceChain};

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct MemChain {
    pairs: Vec<Pair>,
    top: Option<Pair>,
}

impl MemChain {
    /// builds the data structures required to efficiently represent a SourceChain in memory
    /// typically a MemChain should be _mutable_ to facilitate chain.push()
    pub fn new() -> MemChain {
        MemChain {
            pairs: Vec::new(),
            top: None,
        }
    }
}

/// SouceChain trait implementation
impl<'de> SourceChain<'de> for MemChain {
    // appends the current pair to the top of the chain
    fn push(&mut self, entry: &Entry) -> Pair {
        let pair = Pair::new(self, entry);

        if !(pair.validate()) {
            panic!("attempted to push an invalid pair for this source chain");
        }

        let top_pair = self.top();
        let next_pair = pair.header().next().and_then(|h| self.get(&h));
        if top_pair != next_pair {
            // we panic because no code path should attempt to append an invalid pair
            panic!(
                "top pair did not match next hash pair from pushed pair: {:?} vs. {:?}",
                top_pair, next_pair
            );
        }

        // dry run an insertion against a clone and validate the outcome
        let mut validation_chain = self.clone();
        validation_chain.top = Some(pair.clone());
        validation_chain.pairs.insert(0, pair.clone());
        if !validation_chain.validate() {
            // we panic because no code path should ever invalidate the chain
            panic!("adding this pair would invalidate the source chain");
        }

        // @TODO - inserting at the start of a vector is O(n), some other collection could be O(1)
        // @see https://github.com/holochain/holochain-rust/issues/35
        self.top = Some(pair.clone());
        self.pairs.insert(0, pair.clone());

        pair
    }

    fn iter(&self) -> std::slice::Iter<Pair> {
        self.pairs.iter()
    }

    fn validate(&self) -> bool {
        self.pairs.iter().all(|p| p.validate())
    }

    fn get(&self, header_hash: &str) -> Option<Pair> {
        // @TODO - this is a slow way to do a lookup
        // @see https://github.com/holochain/holochain-rust/issues/50
        self.pairs
            .clone()
            .into_iter()
            .find(|p| p.header().hash() == header_hash)
    }

    fn get_entry(&self, entry_hash: &str) -> Option<Pair> {
        // @TODO - this is a slow way to do a lookup
        // @see https://github.com/holochain/holochain-rust/issues/50
        self.pairs
            .clone()
            .into_iter()
            .find(|p| p.entry().hash() == entry_hash)
    }

    fn top(&self) -> Option<Pair> {
        self.top.clone()
    }

    fn top_type(&self, t: &str) -> Option<Pair> {
        self.pairs
            .clone()
            .into_iter()
            .find(|p| p.header().entry_type() == t)
    }
}

// for loop support that consumes chains
impl IntoIterator for MemChain {
    type Item = Pair;
    type IntoIter = std::vec::IntoIter<Pair>;

    fn into_iter(self) -> Self::IntoIter {
        self.pairs.into_iter()
    }
}

// iter() style support for references to chains
impl<'a> IntoIterator for &'a MemChain {
    type Item = &'a Pair;
    type IntoIter = std::slice::Iter<'a, Pair>;

    fn into_iter(self) -> std::slice::Iter<'a, Pair> {
        self.pairs.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::MemChain;
    use chain::{entry::Entry, pair::Pair, SourceChain};
    use serde_json;

    #[test]
    /// tests for MemChain::new()
    fn new() {
        let chain = MemChain::new();

        assert_eq!(None, chain.top());
    }

    #[test]
    /// tests for chain.push()
    fn push() {
        let mut chain = MemChain::new();
        let t = "fooType";

        assert_eq!(None, chain.top());

        // chain top, pair entry and headers should all line up after a push
        let e1 = Entry::new(t, "foo");
        let p1 = chain.push(&e1);

        assert_eq!(Some(p1.clone()), chain.top());
        assert_eq!(e1, p1.entry());
        assert_eq!(e1.hash(), p1.header().entry());

        // we should be able to do it again
        let e2 = Entry::new(t, "bar");
        let p2 = chain.push(&e2);

        assert_eq!(Some(p2.clone()), chain.top());
        assert_eq!(e2, p2.entry());
        assert_eq!(e2.hash(), p2.header().entry());
    }

    #[test]
    /// tests for chain.iter()
    fn iter() {
        let mut chain = super::MemChain::new();

        let t = "fooType";

        let e1 = Entry::new(t, "foo");
        let e2 = Entry::new(t, "bar");
        let e3 = Entry::new(t, "foo");

        let p1 = chain.push(&e1);
        let p2 = chain.push(&e2);
        let p3 = chain.push(&e3);

        // iter() should iterate over references
        assert_eq!(vec![&p3, &p2, &p1], chain.iter().collect::<Vec<&Pair>>());

        // iter() should support functional logic
        assert_eq!(
            vec![&p3, &p1],
            chain
                .iter()
                .filter(|p| p.entry().content() == "foo")
                .collect::<Vec<&Pair>>()
        );
    }

    #[test]
    /// tests for chain.validate()
    fn validate() {
        let mut chain = super::MemChain::new();

        let t = "fooType";

        let e1 = Entry::new(t, "foo");
        let e2 = Entry::new(t, "bar");
        let e3 = Entry::new(t, "baz");

        // for valid pairs its truetles all the way down...
        assert!(chain.validate());

        chain.push(&e1);
        assert!(chain.validate());

        chain.push(&e2);
        assert!(chain.validate());

        chain.push(&e3);
        assert!(chain.validate());
    }

    #[test]
    /// tests for chain.get()
    fn get() {
        let mut chain = super::MemChain::new();

        let t = "fooType";

        let e1 = Entry::new(t, "foo");
        let e2 = Entry::new(t, "bar");
        let e3 = Entry::new(t, "baz");

        let p1 = chain.push(&e1);
        let p2 = chain.push(&e2);
        let p3 = chain.push(&e3);

        assert_eq!(None, chain.get(""));
        assert_eq!(Some(p1.clone()), chain.get(&p1.header().hash()));
        assert_eq!(Some(p2.clone()), chain.get(&p2.header().hash()));
        assert_eq!(Some(p3.clone()), chain.get(&p3.header().hash()));
    }

    #[test]
    /// tests for chain.get_entry()
    fn get_entry() {
        let mut chain = super::MemChain::new();

        let t = "fooType";

        let e1 = Entry::new(t, "foo");
        let e2 = Entry::new(t, "bar");
        let e3 = Entry::new(t, "baz");

        let p1 = chain.push(&e1);
        let p2 = chain.push(&e2);
        let p3 = chain.push(&e3);

        assert_eq!(None, chain.get(""));
        assert_eq!(Some(p1.clone()), chain.get_entry(&p1.entry().hash()));
        assert_eq!(Some(p2.clone()), chain.get_entry(&p2.entry().hash()));
        assert_eq!(Some(p3.clone()), chain.get_entry(&p3.entry().hash()));
    }

    #[test]
    /// tests for chain.top()
    fn top() {
        let mut chain = MemChain::new();
        let t = "fooType";

        let e1 = Entry::new(t, "foo");
        let e2 = Entry::new(t, "bar");

        assert_eq!(None, chain.top());

        let p1 = chain.push(&e1);
        assert_eq!(Some(p1), chain.top());

        let p2 = chain.push(&e2);
        assert_eq!(Some(p2), chain.top());
    }

    #[test]
    /// tests for chain.top_type()
    fn top_type() {
        let mut chain = MemChain::new();
        let t1 = "foo";
        let t2 = "bar";

        // both types start with None
        assert_eq!(None, chain.top_type(t1));
        assert_eq!(None, chain.top_type(t2));

        let e1 = Entry::new(t1, "a");
        let e2 = Entry::new(t2, "b");
        let e3 = Entry::new(t1, "");

        // t1 should be p1
        // t2 should still be None
        let p1 = chain.push(&e1);
        assert_eq!(Some(p1.clone()), chain.top_type(t1));
        assert_eq!(None, chain.top_type(t2));

        // t1 should still be p1
        // t2 should be p2
        let p2 = chain.push(&e2);
        assert_eq!(Some(p1.clone()), chain.top_type(t1));
        assert_eq!(Some(p2.clone()), chain.top_type(t2));

        // t1 should be p3
        // t2 should still be p2
        let p3 = chain.push(&e3);
        assert_eq!(Some(p3.clone()), chain.top_type(t1));
        assert_eq!(Some(p2.clone()), chain.top_type(t2));
    }

    #[test]
    /// tests for IntoIterator implementation
    fn into_iter() {
        let mut chain = super::MemChain::new();

        let t = "fooType";

        let e1 = Entry::new(t, "foo");
        let e2 = Entry::new(t, "bar");
        let e3 = Entry::new(t, "baz");

        let p1 = chain.push(&e1);
        let p2 = chain.push(&e2);
        let p3 = chain.push(&e3);

        // into_iter() by reference
        let mut i = 0;
        let expected = [&p3, &p2, &p1];
        for p in &chain {
            assert_eq!(expected[i], p);
            i = i + 1;
        }

        // do functional things with (&chain).into_iter()
        assert_eq!(
            vec![&p1],
            (&chain)
                .into_iter()
                .filter(|p| p.header().next() == None)
                .collect::<Vec<&Pair>>()
        );

        // into_iter() move
        let mut i = 0;
        let expected = [p3.clone(), p2.clone(), p1.clone()];
        for p in chain.clone() {
            assert_eq!(expected[i], p);
            i = i + 1;
        }
    }

    #[test]
    /// tests for chain serialization round trip through JSON
    fn json_round_trip() {
        let mut chain = super::MemChain::new();

        let t = "foo";
        let e1 = Entry::new(t, "foo");
        let e2 = Entry::new(t, "bar");
        let e3 = Entry::new(t, "baz");

        chain.push(&e1);
        chain.push(&e2);
        chain.push(&e3);

        let json = serde_json::to_string(&chain).unwrap();
        let expected_json = "{\"pairs\":[{\"header\":{\"entry_type\":\"foo\",\"time\":\"\",\"next\":\"QmTDSBLxBhPN8jQzpGjpbopHzfTbjNGmmTBTVSna5icHaX\",\"entry\":\"QmauF2HRPZkF43phNoWmMDqW6hPREXNdT6758PXyBUH9Y1\",\"type_next\":\"QmTDSBLxBhPN8jQzpGjpbopHzfTbjNGmmTBTVSna5icHaX\",\"signature\":\"\"},\"entry\":{\"content\":\"baz\",\"entry_type\":\"foo\"}},{\"header\":{\"entry_type\":\"foo\",\"time\":\"\",\"next\":\"Qma5N19LgwLSY3K2eKteeCjt86QTkxmFCt9JDJfWTrESTy\",\"entry\":\"QmfMjwGasyzX74517w3gL2Be3sozKMGDRwuGJHgs9m6gfS\",\"type_next\":\"Qma5N19LgwLSY3K2eKteeCjt86QTkxmFCt9JDJfWTrESTy\",\"signature\":\"\"},\"entry\":{\"content\":\"bar\",\"entry_type\":\"foo\"}},{\"header\":{\"entry_type\":\"foo\",\"time\":\"\",\"next\":null,\"entry\":\"QmRJzsvyCQyizr73Gmms8ZRtvNxmgqumxc2KUp71dfEmoj\",\"type_next\":null,\"signature\":\"\"},\"entry\":{\"content\":\"foo\",\"entry_type\":\"foo\"}}],\"top\":{\"header\":{\"entry_type\":\"foo\",\"time\":\"\",\"next\":\"QmTDSBLxBhPN8jQzpGjpbopHzfTbjNGmmTBTVSna5icHaX\",\"entry\":\"QmauF2HRPZkF43phNoWmMDqW6hPREXNdT6758PXyBUH9Y1\",\"type_next\":\"QmTDSBLxBhPN8jQzpGjpbopHzfTbjNGmmTBTVSna5icHaX\",\"signature\":\"\"},\"entry\":{\"content\":\"baz\",\"entry_type\":\"foo\"}}}";

        assert_eq!(expected_json, json);
        assert_eq!(chain, serde_json::from_str(&json).unwrap());
    }

}
