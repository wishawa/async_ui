#![deny(unsafe_op_in_unsafe_fn)]
mod control;
mod element;
mod render;
mod unmounting;
mod wrappers;

pub use render::{mount, render};
pub use wrappers::*;
pub mod manual_apis {
    use super::*;
    pub use render::{put_node, render_in_node};
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
