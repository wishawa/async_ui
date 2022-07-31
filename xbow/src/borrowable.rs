use std::rc::Rc;

use crate::{borrow_output::XBowBorrow, EdgeTrait};

pub trait Borrowable {
    type Edge: EdgeTrait;
    fn new(edge: Rc<Self::Edge>) -> Self;
    fn edge(&self) -> &Rc<Self::Edge>;
    fn borrow<'b>(&'b self) -> Option<XBowBorrow<<Self::Edge as EdgeTrait>::BorrowGuard<'b>>> {
        XBowBorrow::new(self.edge().borrow())
    }
    fn borrow_mut<'b>(
        &'b self,
    ) -> Option<XBowBorrow<<Self::Edge as EdgeTrait>::BorrowMutGuard<'b>>> {
        XBowBorrow::new(self.edge().borrow_mut())
    }
}
