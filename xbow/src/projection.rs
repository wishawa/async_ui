use std::rc::Rc;

use crate::{borrow_output::XBowBorrow, in_enum::InEnumNo, EdgeTrait};

pub trait Projection {
    type Edge: EdgeTrait;
    fn new(edge: Rc<Self::Edge>) -> Self;
    fn edge(&self) -> &Rc<Self::Edge>;
    fn invalidate_here_down(&self);
    fn invalidate_up(&self) {
        self.edge().invalidate_up();
    }
}
pub trait ProjectionExt: Projection {
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
impl<T> ProjectionExt for T where T: Projection {}

pub trait ProjectionExtGuaranteed: Projection {
    fn borrow<'b>(&'b self) -> XBowBorrow<'b, <Self::Edge as EdgeTrait>::BorrowGuard<'b>, Self> {
        XBowBorrow::new_without_check(self.edge().borrow(), None)
    }
    fn borrow_mut<'b>(
        &'b self,
    ) -> XBowBorrow<'b, <Self::Edge as EdgeTrait>::BorrowMutGuard<'b>, Self> {
        XBowBorrow::new_without_check(self.edge().borrow_mut(), Some(self))
    }
}
impl<T> ProjectionExtGuaranteed for T
where
    T: Projection,
    T::Edge: EdgeTrait<InEnum = InEnumNo>,
{
}
