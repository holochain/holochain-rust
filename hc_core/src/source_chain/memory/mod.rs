use source_chain;
use std;

#[derive(Clone, Debug, PartialEq)]
pub struct SourceChain {
    pairs: Vec<super::Pair>
}

impl<'a> IntoIterator for &'a SourceChain {
    type Item = super::Pair;
    type IntoIter = std::slice::Iter<'a, super::Pair>;
    fn into_iter (self) -> Self::IntoIter {
        self.pairs.into_iter()
    }
}

impl super::SourceChain for SourceChain {
}
