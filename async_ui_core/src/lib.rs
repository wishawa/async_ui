#![feature(into_future)]
pub mod backend;
pub use mount::mount;
mod children;
mod executor;
mod mount;
mod position;
mod vnode;
pub use children::for_macro;
pub use vnode::VNode;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
