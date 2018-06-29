use std;

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct SourceChain {
    pairs: Vec<super::Pair>,
}

impl SourceChain {
    pub fn new() -> SourceChain {
        SourceChain { pairs: Vec::new() }
    }
}

// for loop support that consumes chains
impl IntoIterator for SourceChain {
    type Item = super::Pair;
    type IntoIter = std::vec::IntoIter<super::Pair>;

    fn into_iter(self) -> Self::IntoIter {
        self.pairs.into_iter()
    }
}

// iter() style support for references to chains
impl<'a> IntoIterator for &'a SourceChain {
    type Item = &'a super::Pair;
    type IntoIter = std::slice::Iter<'a, super::Pair>;

    fn into_iter(self) -> std::slice::Iter<'a, super::Pair> {
        self.pairs.iter()
    }
}

// basic SouceChain trait
impl<'de> super::SourceChain<'de> for SourceChain {
    // appends the current pair to the top of the chain
    fn push(&mut self, pair: &super::Pair) {
        let previous_hash_lookup = pair.header.previous().and_then(|h| self.get(h));

        // smoke test this pair in isolation, and check the hash reference against the top pair
        if !(pair.validate() && self.pairs.first() == previous_hash_lookup.as_ref()) {
            // we panic because no code path should attempt to append an invalid pair
            panic!("attempted to push an invalid pair for this source chain");
        }

        // dry run an insertion against a clone and validate the outcome
        let mut validation_chain = self.clone();
        validation_chain.pairs.insert(0, pair.clone());
        if !validation_chain.validate() {
            // we panic because no code path should ever invalidate the chain
            panic!("adding this pair would invalidate the source chain");
        }

        // @TODO - inserting at the start of a vector is O(n), some other collection could be O(1)
        // @see https://github.com/holochain/holochain-rust/issues/35
        self.pairs.insert(0, pair.clone())
    }

    fn iter(&self) -> std::slice::Iter<super::Pair> {
        self.pairs.iter()
    }

    fn validate(&self) -> bool {
        self.pairs.iter().all(|p| p.validate())
    }

    fn get(&self, header_hash: u64) -> Option<super::Pair> {
        // @TODO - this is a slow way to do a lookup
        // @see https://github.com/holochain/holochain-rust/issues/50
        self.pairs
            .clone()
            .into_iter()
            .find(|p| p.header.hash() == header_hash)
    }

    fn get_entry(&self, entry_hash: u64) -> Option<super::Pair> {
        // @TODO - this is a slow way to do a lookup
        // @see https://github.com/holochain/holochain-rust/issues/50
        self.pairs
            .clone()
            .into_iter()
            .find(|p| p.entry.hash() == entry_hash)
    }
}

#[cfg(test)]
mod tests {
    use common::entry::{Entry, Header};
    use serde_json;
    use source_chain::{Pair, SourceChain};

    // helper to spin up pairs for testing
    // @TODO - do we want to expose something like this as a general utility?
    // @see https://github.com/holochain/holochain-rust/issues/34
    fn test_pair(previous_pair: Option<&Pair>, s: &str) -> Pair {
        let e = Entry::new(&s.to_string());
        let previous = match previous_pair {
            Some(p) => Some(p.header.hash()),
            None => None,
        };
        let h = Header::new(previous, &e);
        Pair::new(&h, &e)
    }

    #[test]
    fn validate() {
        let p1 = test_pair(None, "foo");
        let p2 = test_pair(Some(&p1), "bar");

        // for valid pairs its truetles all the way down...
        let mut chain = super::SourceChain::new();
        assert!(chain.validate());
        chain.push(&p1);
        assert!(chain.validate());
        chain.push(&p2);
        assert!(chain.validate());
    }

    #[test]
    fn get() {
        let p1 = test_pair(None, "foo");
        let p2 = test_pair(Some(&p1), "bar");
        let p3 = test_pair(Some(&p2), "baz");

        let mut chain = super::SourceChain::new();
        chain.push(&p1);
        chain.push(&p2);
        chain.push(&p3);

        assert_eq!(None, chain.get(0));
        assert_eq!(Some(p1.clone()), chain.get(p1.header.hash()));
        assert_eq!(Some(p2.clone()), chain.get(p2.header.hash()));
        assert_eq!(Some(p3.clone()), chain.get(p3.header.hash()));
    }

    #[test]
    fn get_entry() {
        let p1 = test_pair(None, "foo");
        let p2 = test_pair(Some(&p1), "bar");
        let p3 = test_pair(Some(&p2), "baz");

        let mut chain = super::SourceChain::new();
        chain.push(&p1);
        chain.push(&p2);
        chain.push(&p3);

        assert_eq!(None, chain.get(0));
        assert_eq!(Some(p1.clone()), chain.get_entry(p1.entry.hash()));
        assert_eq!(Some(p2.clone()), chain.get_entry(p2.entry.hash()));
        assert_eq!(Some(p3.clone()), chain.get_entry(p3.entry.hash()));
    }

    #[test]
    fn valid_push() {
        let p1 = test_pair(None, "foo");
        let p2 = test_pair(Some(&p1), "bar");

        let mut chain = super::SourceChain::new();
        chain.push(&p1);
        chain.push(&p2);
    }

    #[test]
    #[should_panic(expected = "attempted to push an invalid pair for this source chain")]
    fn invalid_push() {
        let p1 = test_pair(None, "foo");
        let p2 = test_pair(Some(&p1), "bar");

        let mut chain = super::SourceChain::new();

        // wrong order, must panic!
        chain.push(&p2);
        chain.push(&p1);
    }

    #[test]
    fn iter() {
        // setup
        let p1 = test_pair(None, "foo");
        let p2 = test_pair(Some(&p1), "bar");
        let p3 = test_pair(Some(&p2), "foo");

        let mut chain = super::SourceChain::new();
        chain.push(&p1);
        chain.push(&p2);
        chain.push(&p3);

        // iter() should iterate over references
        assert_eq!(vec![&p3, &p2, &p1], chain.iter().collect::<Vec<&Pair>>());

        // iter() should support functional logic
        assert_eq!(
            vec![&p3, &p1],
            chain
                .iter()
                .filter(|p| p.entry.content() == "foo")
                .collect::<Vec<&Pair>>()
        );
    }

    #[test]
    fn into_iter() {
        // setup
        let p1 = test_pair(None, "foo");
        let p2 = test_pair(Some(&p1), "bar");
        let p3 = test_pair(Some(&p2), "baz");

        let mut chain = super::SourceChain::new();
        chain.push(&p1);
        chain.push(&p2);
        chain.push(&p3);

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
                .filter(|p| p.header.previous() == None)
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
    fn json_round_trip() {
        // setup
        let p1 = test_pair(None, "foo");
        let p2 = test_pair(Some(&p1), "bar");
        let p3 = test_pair(Some(&p2), "baz");

        let mut chain = super::SourceChain::new();
        chain.push(&p1);
        chain.push(&p2);
        chain.push(&p3);

        let json = serde_json::to_string(&chain).unwrap();
        let expected_json = "{\"pairs\":[{\"header\":{\"previous\":14317484463802884792,\"entry\":16260972211344176173,\"hash\":4531740482513330668},\"entry\":{\"content\":\"baz\",\"hash\":16260972211344176173}},{\"header\":{\"previous\":2931328680099981702,\"entry\":3676438629107045207,\"hash\":14317484463802884792},\"entry\":{\"content\":\"bar\",\"hash\":3676438629107045207}},{\"header\":{\"previous\":null,\"entry\":4506850079084802999,\"hash\":2931328680099981702},\"entry\":{\"content\":\"foo\",\"hash\":4506850079084802999}}]}";

        assert_eq!(expected_json, json);
        assert_eq!(chain, serde_json::from_str(&json).unwrap());
    }
}
