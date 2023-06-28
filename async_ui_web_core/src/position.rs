/*!
For ordering DOM nodes relative to each other.

When multiple futures are combined (by join, race, etc.), we want to render
them in order. For example, this arrangement of futures

```text

                           ┌───────┐
                           │       │
                           │ <div> │
                           │       │
                           └───┬───┘
                               │
                               │
         ┌─────────────────────┴─────────────────────┐
         │ Join Future                               │
         │                                           │
         │ Child 1         Child 2         Child 3   │
         │ ┌───────┐       ┌───────┐       ┌───────┐ │
         │ │       │       │       │       │       │ │
         │ │       │       │<label>│       │<input>│ │
         │ │       │       │       │       │       │ │
         │ └───┬───┘       └───────┘       └───────┘ │
         │     │                                     │
         └─────┼─────────────────────────────────────┘
               │
               │
               │
 ┌─────────────┴──────────────┐
 │ Race Future                │
 │                            │
 │ Child 1         Child 2    │
 │ ┌───────┐       ┌───────┐  │
 │ │       │       │       │  │
 │ │ <nav> │       │ <img> │  │
 │ │       │       │       │  │
 │ └───────┘       └───────┘  │
 │                            │
 │                            │
 └────────────────────────────┘

```

should render to
```html
<div>
    <nav />
    <img />
    <label />
    <input />
</div>
```

To acheive this, the combinators (join/race/...) give each child its index ([PositionSegment]).
When a future wants to insert something, the indices are assembled into a path ([ChildPosition]).
Paths are ordered and stored in a [BTreeMap][std::collections::BTreeMap], so we find the rendered element
with the next higher path and `insertBefore` that element.
*/

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

/// An index that combinators give their children.
type PositionSegment = u32;

/// A path assembled from indices.
///
/// This path is assembled from the leaf up the tree, thus the most significant
/// segment is the last item in it.
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
    /// Add a segment to the path.
    /// Remember that later segments are more significant.
    pub fn wrap(&mut self, index: PositionSegment) {
        self.0.push(index);
    }
    /// Get the path with the first (least significant) segment incremented by 1.
    /// This is for using [std::collection::BTreeMap::range].
    ///
    /// Panics if the path is empty.
    /// In that case the next sibling would be [+∞],
    /// but we handle that separately in [crate::context] anyway.
    pub fn next_sibling(&self) -> Self {
        let mut content = self.0.clone();
        content[0] += 1;
        Self(content)
    }
    /// Check if the path is empty.
    pub fn is_root(&self) -> bool {
        self.0.is_empty()
    }
}
