use std::marker::PhantomData;

use observables::{Listenable, ObservableAs};

pub fn dummy_handler<T>() -> &'static mut dyn FnMut(T) {
    Box::leak(Box::new(|_| {}))
}
pub fn is_dummy_handler<T: 'static>(h: &mut dyn FnMut(T)) -> bool {
    std::ptr::eq(
        h as *const dyn FnMut(T),
        dummy_handler::<T>() as *const dyn FnMut(T),
    )
}

pub struct DummyObservableAs<T>(pub PhantomData<T>);
const DUMMY_USED: &str = "dummy prop used";
impl<T> Listenable for DummyObservableAs<T> {
    fn add_waker(&self, _waker: std::task::Waker) {
        panic!("{}", DUMMY_USED)
    }
    fn get_version(&self) -> observables::Version {
        panic!("{}", DUMMY_USED)
    }
}
impl<T> ObservableAs<T> for DummyObservableAs<T> {
    fn visit_dyn_as(&self, _visitor: &mut dyn FnMut(&T)) {
        panic!("{}", DUMMY_USED)
    }
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
