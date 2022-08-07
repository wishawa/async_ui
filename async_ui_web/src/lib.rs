use async_ui_core::render::Render as RenderBase;
use backend::Backend;

pub mod backend;
pub mod executor;
mod mount;
pub use mount::{mount, mount_at};

pub type Render<'c> = RenderBase<'c, Backend>;

pub mod __private_macro_only {
    pub use super::Render;
    pub use async_ui_core::render as render_base;
    #[macro_export]
    macro_rules! children {
        [$($ch:expr),*] => {
            ({
                let children: $crate::__private_macro_only::Render = $crate::__private_macro_only::render_base![
                    $($ch),*
                ];
                children
            })
        };
    }
}

#[cfg(test)]
mod tests {
    use super::children;

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
        let _a = children![async {}];
    }
}
