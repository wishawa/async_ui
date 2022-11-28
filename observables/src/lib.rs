use std::{borrow::Borrow, task::Waker};

use transformers::{for_each::ForEach, map::Map};
pub use version::Version;
mod impls;
mod next_change;
mod transformers;
mod version;
pub use next_change::NextChangeFuture;

#[cfg(feature = "async-channel")]
pub mod async_channel;
pub mod cell;
#[cfg(feature = "futures-signals")]
pub mod futures_signals;

pub trait Listenable {
    fn add_waker(&self, waker: Waker);
    fn get_version(&self) -> Version;
}

pub trait ObservableBase: Listenable {
    type Data: ?Sized;
    fn visit_base<'b, F: FnOnce(&Self::Data) -> U, U>(&'b self, f: F) -> U;
}

pub trait ObservableAs<Z: ?Sized>: Listenable {
    fn visit_dyn_as(&self, visitor: &mut dyn FnMut(&Z));
    #[allow(non_snake_case)]
    fn __import_ObservableAsExt_for_methods(&self) {}
}
pub trait ObservableAsExt<Z: ?Sized>: ObservableAs<Z> {
    fn visit<U, F: FnOnce(&Z) -> U>(&self, visitor: F) -> U {
        enum State<U, F> {
            Func(F),
            Result(U),
            Null,
        }
        let mut state = State::Func(visitor);
        self.visit_dyn_as(&mut |v| match std::mem::replace(&mut state, State::Null) {
            State::Func(f) => state = State::Result(f(v)),
            _ => panic!("already visited"),
        });
        match state {
            State::Result(u) => u,
            _ => panic!("visitor function not called"),
        }
    }
    fn get(&self) -> Z::Owned
    where
        Z: ToOwned,
    {
        self.visit(ToOwned::to_owned)
    }
    fn map<O, M>(self, mapper: M) -> Map<Self, Z, O, M>
    where
        M: Fn(&Z) -> O,
        Self: Sized,
    {
        Map::new(self, mapper)
    }
    fn until_change<'i>(&'i self) -> NextChangeFuture<Self, &'i Self> {
        NextChangeFuture::new(self)
    }
    fn for_each<H>(self, handler: H) -> ForEach<Self, Z, H>
    where
        H: FnMut(&Z),
        Self: Sized,
    {
        ForEach::new(self, handler)
    }
    // fn for_each_async<H, F>(
    //     &self,
    //     handler: H,
    // ) -> ForEachAsync<Self, H, F>
    // where
    //     H: FnMut(&Z) -> F,
    //     F: Future<Output = ()>,
    //     Self: Sized
    // {
    // }
}
impl<Z, O> ObservableAs<Z> for O
where
    Z: ?Sized,
    O: ObservableBase + ?Sized,
    O::Data: Borrow<Z>,
{
    fn visit_dyn_as(&self, visitor: &mut dyn FnMut(&Z)) {
        self.visit_base(|val| visitor(val.borrow()));
    }
}

// struct ObservableWrapper<'a, O: ?Sized, Z>{
//     pub data: &'a O,
//     _phantom: PhantomData<Z>
// }

// impl<'a, O, Z> Listenable for ObservableWrapper<'a, O, Z>
// where
//     O: Listenable + ?Sized
// {
//     fn add_waker(&self, waker: Waker) {
//         <O as Listenable>::add_waker(self.0, waker)
//     }
//     fn get_version(&self) -> Version {
//         <O as Listenable>::get_version(self.0)
//     }
// }

// impl<'a, O, Z> ObservableBase for ObservableWrapper<'a, O, Z>
// where
//     O: ObservableBase + ?Sized
// {
//     type Data = Z;

//     fn visit_base<'b, F: FnOnce(&Self::Data) -> U, U>(&'b self, f: F) -> U {
//         <O as ObservableBase>::visit_base(self.0, f)
//     }
// }

impl<Z, O> ObservableAsExt<Z> for O
where
    Z: ?Sized,
    O: ObservableAs<Z> + ?Sized,
{
}

impl<'a, Z: ?Sized> Listenable for &'a dyn ObservableAs<Z> {
    fn add_waker(&self, waker: Waker) {
        (*self).add_waker(waker)
    }
    fn get_version(&self) -> Version {
        (*self).get_version()
    }
}

impl<'a, Z: ?Sized> ObservableBase for &'a dyn ObservableAs<Z> {
    type Data = Z;

    fn visit_base<'b, F: FnOnce(&Self::Data) -> U, U>(&'b self, f: F) -> U {
        (*self).visit(f)
    }
}
