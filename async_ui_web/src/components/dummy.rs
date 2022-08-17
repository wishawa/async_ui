pub fn dummy_handler<T>(_arg: T) {}
pub fn is_dummy<'c, T: 'c>(func: &(dyn Fn(T) + 'c)) -> bool {
    std::ptr::eq(
        func as *const _,
        &dummy_handler::<T> as &(dyn Fn(T) + 'c) as *const _,
    )
}
