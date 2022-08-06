use std::task::Waker;

pub mod cell;

pub trait ObservableBase {
    fn add_waker(&self, waker: Waker);
    fn get_version(&self) -> u64;
}
pub trait Observable<T>: ObservableBase {
    fn visit<R, F: FnOnce(&T) -> R>(&self, func: F) -> R;
}
pub trait Mutatable<T> {
    fn visit_mut<R, F: FnOnce(&mut T) -> R>(&self, func: F) -> R;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
