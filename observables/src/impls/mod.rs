use crate::{Observable, ObservableBase, Version};
mod stdlib;

pub struct NoChange<T>(pub T);
impl<T> ObservableBase for NoChange<T> {
    fn add_waker(&self, _waker: std::task::Waker) {
        // NO-OP
    }
    fn get_version(&self) -> crate::Version {
        Version::new()
    }
}
impl<T> Observable for NoChange<T> {
    type Data = T;
    fn visit<R, F: FnOnce(&Self::Data) -> R>(&self, func: F) -> R {
        func(&self.0)
    }
}

impl<'t, T> ObservableBase for &'t T
where
    T: ObservableBase,
{
    fn add_waker(&self, waker: std::task::Waker) {
        <T as ObservableBase>::add_waker(self, waker)
    }
    fn get_version(&self) -> Version {
        <T as ObservableBase>::get_version(self)
    }
}
impl<'t, T> Observable for &'t T
where
    T: Observable,
{
    type Data = T::Data;

    fn visit<R, F: FnOnce(&Self::Data) -> R>(&self, func: F) -> R {
        <T as Observable>::visit(self, func)
    }
}
