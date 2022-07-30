use async_ui_core::children::Children as ChildrenBase;
use backend::Backend;

pub mod backend;
pub mod executor;

pub type Children<'c> = ChildrenBase<'c, Backend>;

pub mod __for_macro {
    pub use super::Children;
    pub use async_ui_core::children as children_base;
    #[macro_export]
    macro_rules! children {
        [$($ch:expr),*] => {
            ({
                let children: $crate::__for_macro::Children = $crate::__for_macro::children_base![
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
        async fn test(a: &str) {}
        fn test_blocking(a: &str) {}
        let b = String::from("hola");
        let f = test(&b);
        let f2 = test_blocking(&String::from("haha"));
        let _ = async {
            test(&String::from("hi")).await;
        };
    }
}
