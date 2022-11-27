use crate::{Listenable, ObservableBase, ObservableBorrow, Version};

impl<T> Listenable for [T; 1] {
    fn add_waker(&self, _waker: std::task::Waker) {
        // NO-OP
    }
    fn get_version(&self) -> crate::Version {
        Version::new()
    }
}
impl<T> ObservableBase for [T; 1] {
    type Data = T;
    fn borrow_observable<'b>(&'b self) -> ObservableBorrow<'b, T> {
        ObservableBorrow::Borrow(&self[0])
    }
}
