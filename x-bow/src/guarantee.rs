use std::cell::Ref;

use crate::{borrow_mut_guard::BorrowMutGuard, path_ext::PathExt, Path};

pub trait PathExtGuaranteed: PathExt {
    fn borrow<'d>(&'d self) -> Ref<'d, <Self as Path>::Out>
    where
        Self: 'd,
    {
        self.borrow_opt().unwrap()
    }
    fn borrow_mut<'d>(&'d self) -> BorrowMutGuard<'d, <Self as Path>::Out>
    where
        Self: 'd,
    {
        self.borrow_opt_mut().unwrap()
    }
}
