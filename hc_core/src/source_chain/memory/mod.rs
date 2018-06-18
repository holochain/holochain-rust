use std;

#[derive(Clone, Debug, PartialEq)]
pub struct SourceChain {
    pairs: Vec<super::Pair>
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
        self.pairs.push(pair.clone())
    }
}

#[cfg(test)]
mod tests {
    use common;
    use source_chain::Pair;
    use source_chain::SourceChain;

    #[test]
    fn round_trip() {
        let mut chain = super::SourceChain { pairs: Vec::new() };

        let e1 = common::entry::Entry {};
        let h1 = common::entry::Header::new(None, &e1);
        let p1 = &Pair::new(h1, e1);
        chain.push(p1);

        for pair in chain {
            println!("{:?}", pair)
        }
    }
}
