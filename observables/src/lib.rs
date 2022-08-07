use std::{pin::Pin, task::Waker};

use transformers::map::Map;
mod transformers;

pub mod cell;
#[cfg(feature = "futures-signals")]
mod futures_signals;

pub trait ObservableBase {
    fn add_waker(self: Pin<&Self>, waker: Waker);
    fn get_version(self: Pin<&Self>) -> u64;
}
pub trait Observable: ObservableBase {
    type Data;
    fn visit<R, F: FnOnce(&Self::Data) -> R>(&self, func: F) -> R;
}

pub trait ObservableExt: Observable {
    fn map<O, M>(self, mapper: M) -> Map<Self, O, M>
    where
        M: Fn(&Self::Data) -> O,
        Self: Sized,
    {
        Map::new(self, mapper)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
