use async_ui_core::fragment::Fragment as FragmentBase;
use backend::Backend;
mod backend;
pub mod components;
mod executor;
mod mount;
mod widget;
pub use gtk;
pub use mount::{mount, mount_at};

pub use futures_lite;

pub type Fragment<'c> = FragmentBase<'c, Backend>;

pub mod __private_macro_only {
    pub use super::Fragment;
    pub use async_ui_core::fragment as fragment_base;
    #[macro_export]
    macro_rules! fragment {
        [$($ch:expr),* $(,)?] => {
            ::std::convert::identity::<$crate::__private_macro_only::Fragment>($crate::__private_macro_only::fragment_base![
                $($ch),*
            ])
        };
    }
}

pub fn fragment<'c, T: Into<Fragment<'c>>>(tuple: T) -> Fragment<'c> {
    tuple.into()
}
