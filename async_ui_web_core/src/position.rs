use smallvec::SmallVec;
use std::cmp::Ordering;

/// Reverse lexicographical comparison.
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

type PositionSegment = u32;

#[derive(Clone, PartialEq, Eq, Hash, Default, Debug)]
pub struct ChildPosition(SmallVec<[PositionSegment; 4]>);
impl PartialOrd for ChildPosition {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for ChildPosition {
    fn cmp(&self, other: &Self) -> Ordering {
        compare_slice_reversed(&self.0, &other.0)
    }
}
impl ChildPosition {
    pub fn wrap(&mut self, index: PositionSegment) {
        self.0.push(index);
    }
    pub fn next_sibling(&self) -> Self {
        let mut content = self.0.clone();
        content[0] += 1;
        Self(content)
    }
    pub fn is_root(&self) -> bool {
        self.0.is_empty()
    }
}
