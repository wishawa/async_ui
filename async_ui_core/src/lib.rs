#![feature(into_future)]
pub mod backend;
pub use mount::mount;
pub mod children;
mod executor;
pub mod mount;
mod position;
pub mod vnode;
pub use children::__for_macro;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
