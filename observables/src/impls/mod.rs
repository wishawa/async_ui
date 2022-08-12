use crate::{Observable, ObservableBase, ObservableBorrowed, Version};
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
    fn obs_borrow<'b>(&'b self) -> ObservableBorrowed<'b, Self::Data> {
        ObservableBorrowed::Ref(&self.0)
    }
}

// impl<'t, T: ?Sized> ObservableBase for &'t T
// where
//     T: ObservableBase,
// {
//     fn add_waker(&self, waker: std::task::Waker) {
//         <T as ObservableBase>::add_waker(self, waker)
//     }
//     fn get_version(&self) -> Version {
//         <T as ObservableBase>::get_version(self)
//     }
// }
// impl<'t, T: ?Sized> Observable for &'t T
// where
//     T: Observable,
// {
//     type Data = T::Data;

//     fn obs_borrow<'b>(&'b self) -> ObservableBorrowed<'b, Self::Data> {
//         <T as Observable>::obs_borrow(self)
//     }
// }
