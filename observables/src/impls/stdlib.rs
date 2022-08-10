use std::borrow::Cow;

use crate::{Observable, ObservableBase, Version};

macro_rules! impl_base_inner {
    () => {
        fn add_waker(&self, _waker: std::task::Waker) {
            // NO-OP
        }
        fn get_version(&self) -> crate::Version {
            Version::new()
        }
    };
}
macro_rules! impl_base_primitive {
    ($primitive:ty) => {
        impl ObservableBase for $primitive {
            impl_base_inner!();
        }
    };
}
macro_rules! impl_primitive {
    ($primitive:ty) => {
        impl_primitive!($primitive, $primitive);
    };
    ($primitive:ty, $derefto:ty) => {
        impl_base_primitive!($primitive);
        impl Observable for $primitive {
            type Data = $derefto;
            fn visit<R, F: FnOnce(&Self::Data) -> R>(&self, func: F) -> R {
                func(self)
            }
        }
    };
}
impl_primitive!(bool);
impl_primitive!(char);
impl_primitive!(f32);
impl_primitive!(f64);
impl_primitive!(i128);
impl_primitive!(i16);
impl_primitive!(i32);
impl_primitive!(i64);
impl_primitive!(i8);
impl_primitive!(isize);
impl_primitive!(u128);
impl_primitive!(u16);
impl_primitive!(u32);
impl_primitive!(u64);
impl_primitive!(u8);
impl_primitive!(usize);
impl_primitive!(String, str);

impl<'a, T: Clone> ObservableBase for Cow<'a, T> {
    impl_base_inner!();
}
impl<'a, T: Clone> Observable for Cow<'a, T> {
    type Data = T;
    fn visit<R, F: FnOnce(&Self::Data) -> R>(&self, func: F) -> R {
        match self {
            Cow::Borrowed(borrowed) => func(*borrowed),
            Cow::Owned(owned) => func(owned),
        }
    }
}

impl<'a> ObservableBase for &'a str {
    impl_base_inner!();
}
impl<'a> Observable for &'a str {
    type Data = str;

    fn visit<R, F: FnOnce(&Self::Data) -> R>(&self, func: F) -> R {
        func(self)
    }
}
