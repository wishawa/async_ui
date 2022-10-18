use std::{future::IntoFuture, rc::Rc};

use async_ui_core::fragment::Fragment as FragmentBase;
use backend::Backend;

pub mod backend;
pub mod components;
pub mod executor;
mod mount;
pub mod utils;
mod window;
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

pub async fn with_context<I: IntoFuture, T: 'static>(future: I, value: Rc<T>) -> I::Output {
    use async_ui_core::vnode::node_context::WithContext;
    WithContext::<Backend, _>::new(future.into_future(), value).await
}
pub fn get_context<T: 'static>() -> Rc<T> {
    async_ui_core::vnode::node_context::get_context::<Backend, T>()
}

#[cfg(test)]
mod tests {
    use super::fragment;

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
        async fn test(_a: &str) {}
        fn test_blocking(_a: &str) {}
        let b = String::from("hola");
        let _f = test(&b);
        let _f2 = test_blocking(&String::from("haha"));
        let _ = async {
            test(&String::from("hi")).await;
        };
        let _a = fragment![async {}];
    }
}
