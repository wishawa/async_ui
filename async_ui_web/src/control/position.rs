use smallvec::SmallVec;
use std::cmp::Ordering;
type PositionIndex = usize;

#[derive(Clone, PartialEq, Eq, Hash, Default)]
pub(crate) struct PositionIndices(SmallVec<[PositionIndex; 8]>);
impl PartialOrd for PositionIndices {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for PositionIndices {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}
impl PositionIndices {
    pub fn merge(mut self, other: Self) -> Self {
        self.0.extend(other.0.into_iter());
        self
    }
    pub fn nest(&mut self, index: PositionIndex) {
        self.0.push(index);
    }
}
