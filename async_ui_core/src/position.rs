use smallvec::SmallVec;
use std::cmp::Ordering;

fn compare_slice_reversed<C: Ord>(s1: &[C], s2: &[C]) -> Ordering {
    for (e1, e2) in s1.iter().rev().zip(s2.iter().rev()) {
        match e1.cmp(e2) {
            Ordering::Equal => {}
            cmp_res => return cmp_res,
        }
    }
    let (l1, l2) = (s1.len(), s2.len());
    l1.cmp(&l2)
}

type PositionSegment = usize;

#[derive(Clone, PartialEq, Eq, Hash, Default, Debug)]
pub struct PositionIndex(SmallVec<[PositionSegment; 4]>);
impl PartialOrd for PositionIndex {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for PositionIndex {
    fn cmp(&self, other: &Self) -> Ordering {
        compare_slice_reversed(&self.0, &other.0)
    }
}
impl PositionIndex {
    pub fn merge(mut self, other: Self) -> Self {
        self.0.extend(other.0.into_iter());
        self
    }
    pub fn nest(&mut self, index: PositionSegment) {
        self.0.push(index);
    }
}
