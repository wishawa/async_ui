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
