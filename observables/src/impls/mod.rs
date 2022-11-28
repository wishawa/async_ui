use crate::{Listenable, ObservableBase, Version};

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

    fn visit_base<'b, F: FnOnce(&Self::Data) -> U, U>(&'b self, f: F) -> U {
        f(&self[0])
    }
}
