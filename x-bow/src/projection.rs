use std::rc::Rc;

use crate::{borrow_output::XBowBorrow, edge::EdgeTrait, in_enum::InEnumNo};

pub trait Tracked {
    type Edge: EdgeTrait;
    fn new(edge: Rc<Self::Edge>) -> Self;
    fn edge(&self) -> &Rc<Self::Edge>;
    fn invalidate_here_down(&self);
    fn invalidate_up(&self) {
        self.edge().invalidate_up();
    }
}
pub trait TrackedExt: Tracked {
    fn borrow_opt<'b>(
        &'b self,
    ) -> Option<XBowBorrow<'b, <Self::Edge as EdgeTrait>::BorrowGuard<'b>, Self>> {
        XBowBorrow::new(self.edge().borrow(), None)
    }
    fn borrow_mut_opt<'b>(
        &'b self,
    ) -> Option<XBowBorrow<'b, <Self::Edge as EdgeTrait>::BorrowMutGuard<'b>, Self>> {
        XBowBorrow::new(self.edge().borrow_mut(), Some(self))
    }
}
impl<T> TrackedExt for T where T: Tracked {}

pub trait TrackedExtGuaranteed: Tracked {
    fn borrow<'b>(&'b self) -> XBowBorrow<'b, <Self::Edge as EdgeTrait>::BorrowGuard<'b>, Self> {
        XBowBorrow::new_without_check(self.edge().borrow(), None)
    }
    fn borrow_mut<'b>(
        &'b self,
    ) -> XBowBorrow<'b, <Self::Edge as EdgeTrait>::BorrowMutGuard<'b>, Self> {
        XBowBorrow::new_without_check(self.edge().borrow_mut(), Some(self))
    }
}
impl<T> TrackedExtGuaranteed for T
where
    T: Tracked,
    T::Edge: EdgeTrait<InEnum = InEnumNo>,
{
}
