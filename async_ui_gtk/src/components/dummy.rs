pub(super) fn dummy_handler<T>() -> &'static mut dyn FnMut(T) {
    Box::leak(Box::new(|_| {}))
}
pub(super) fn is_dummy_handler<T: 'static>(h: &mut dyn FnMut(T)) -> bool {
    std::ptr::eq(
        h as *const dyn FnMut(T),
        dummy_handler::<T>() as *const dyn FnMut(T),
    )
}

#[cfg(test)]
mod tests {
    use super::{dummy_handler, is_dummy_handler};

    #[test]
    fn test_size() {
        let v = super::dummy_handler::<i32>();
        assert_eq!(0, std::mem::size_of_val(v));
        assert_eq!(0, std::mem::size_of_val(&*Box::new(|_: i32| {})));
    }
    #[test]
    fn test_equality() {
        assert!(is_dummy_handler(dummy_handler::<i32>()));
        assert!(!is_dummy_handler(&mut |_: u64| {}));
        assert!(!is_dummy_handler(Box::leak(Box::new(|_: u64| {}))));
        assert!(!is_dummy_handler(Box::leak(Box::new(|_: i32| {}))));
    }
}
