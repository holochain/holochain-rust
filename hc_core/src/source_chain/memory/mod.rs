use std;

#[derive(Clone, Debug, PartialEq)]
pub struct SourceChain {
    pairs: Vec<super::Pair>
}

impl SourceChain {
    pub fn new(pairs: &Vec<super::Pair>) -> SourceChain {
        SourceChain {
            pairs: pairs.clone(),
        }
    }
}

impl IntoIterator for SourceChain {
    type Item = super::Pair;
    type IntoIter = std::vec::IntoIter<super::Pair>;
    fn into_iter (self) -> Self::IntoIter {
        self.pairs.into_iter()
    }
}

impl super::SourceChain for SourceChain {
    fn push (&mut self, pair: &super::Pair) {
        self.pairs.insert(0, pair.clone())
    }
}

#[cfg(test)]
mod tests {
    use common::entry::Entry;
    use common::entry::Header;
    use source_chain::Pair;
    use source_chain::SourceChain;

    #[test]
    fn round_trip() {
        let mut chain = super::SourceChain::new(&Vec::new());

        let e1 = Entry::new(&String::from("some content"));
        let h1 = Header::new(None, &e1);
        assert_eq!(h1.entry(), e1.hash());
        assert_eq!(h1.previous(), None);

        let p1 = Pair::new(&h1, &e1);
        chain.push(&p1);

        let mut iter1 = chain.clone().into_iter();
        let i = iter1.next().unwrap();
        assert_eq!(p1, i);

        let e2 = Entry::new(&String::from("some more content"));
        let h2 = Header::new(Some(h1.hash()), &e2);
        assert_eq!(h2.entry(), e2.hash());
        assert_eq!(h2.previous().unwrap(), h1.hash());

        let p2 = Pair::new(&h2, &e2);
        chain.push(&p2);

        let mut iter2 = chain.clone().into_iter();
        let i = iter2.next().unwrap();
        assert_eq!(p2, i);
        let i2 = iter2.next().unwrap();
        assert_eq!(p1, i2);

        let iter3 = chain.clone().into_iter();
        let f = iter3.filter(|p| p.entry.content() == "some content").last().unwrap();
        assert_eq!(f, p1)
    }
}
