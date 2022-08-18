pub fn dummy_handler<T>(_arg: T) {}
pub(super) fn create_dummy<'c, T: 'c>() -> &'c mut (dyn FnMut(T) + 'c) {
    // Box does not allocate for zero-sized values
    let boxed = Box::new(dummy_handler) as Box<dyn FnMut(T) + 'c>;
    Box::leak(boxed)
}
pub fn is_dummy<'c, T: 'c>(func: &mut (dyn FnMut(T) + 'c)) -> bool {
    std::ptr::eq(
        func as *const dyn FnMut(T),
        create_dummy() as *const dyn FnMut(T),
    )
}

#[cfg(test)]
mod tests {
    use super::{create_dummy, is_dummy};

    #[test]
    fn test_dummy_size() {
        assert_eq!(std::mem::size_of_val(create_dummy::<i32>()), 0);
    }
    #[test]
    fn test_compare() {
        let d1 = create_dummy::<i32>();
        let d2 = create_dummy::<i32>();
        let d3 = create_dummy::<i32>();
        let r1 = &mut |_v: i32| {};
        assert!(is_dummy(d1));
        assert!(is_dummy(d2));
        assert!(is_dummy(d3));
        assert!(!is_dummy(r1));
    }
}
