use std::rc::Rc;

use crate::{bool_type::False, borrow_output::XBowBorrow, EdgeTrait};

pub trait Borrowable {
    type Edge: EdgeTrait;
    fn new(edge: Rc<Self::Edge>) -> Self;
    fn edge(&self) -> &Rc<Self::Edge>;
    fn borrow_opt<'b>(&'b self) -> Option<XBowBorrow<<Self::Edge as EdgeTrait>::BorrowGuard<'b>>> {
        XBowBorrow::new(self.edge().borrow())
    }
    fn borrow_mut_opt<'b>(
        &'b self,
    ) -> Option<XBowBorrow<<Self::Edge as EdgeTrait>::BorrowMutGuard<'b>>> {
        XBowBorrow::new(self.edge().borrow_mut())
    }
}

pub trait BorrowableGuaranteed: Borrowable {
    fn borrow<'b>(&'b self) -> XBowBorrow<<Self::Edge as EdgeTrait>::BorrowGuard<'b>> {
        XBowBorrow::new_without_check(self.edge().borrow())
    }
    fn borrow_mut<'b>(&'b self) -> XBowBorrow<<Self::Edge as EdgeTrait>::BorrowMutGuard<'b>> {
        XBowBorrow::new_without_check(self.edge().borrow_mut())
    }
}
impl<T> BorrowableGuaranteed for T
where
    T: Borrowable,
    T::Edge: EdgeTrait<InEnum = False>,
{
}
