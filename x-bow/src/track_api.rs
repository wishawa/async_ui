use std::cell::{Ref, RefMut};

use crate::{is_guaranteed::IsGuaranteed, node_down::NodeDownTrait, until_change::UntilChange};

pub trait Tracked<'u, T> {
    fn borrow_opt<'b>(&'b self) -> Option<Ref<'b, T>>
    where
        'u: 'b;
    fn borrow_opt_mut<'b>(&'b self) -> Option<RefMut<'b, T>>
    where
        'u: 'b,
    {
        self.borrow_opt_mut_custom(true, true, true)
    }
    fn borrow_opt_mut_custom<'b>(
        &'b self,
        wake_outside: bool,
        wake_here: bool,
        wake_inside: bool,
    ) -> Option<RefMut<'b, T>>
    where
        'u: 'b;
    fn until_change_full<'a>(&'a self, outside: bool, here: bool, inside: bool) -> UntilChange<'a>
    where
        'u: 'a;
    fn until_change<'a>(&'a self) -> UntilChange<'a>
    where
        'u: 'a,
    {
        self.until_change_full(true, true, false)
    }
}

pub trait TrackedGuaranteed<'u, T>: Tracked<'u, T>
where
    T: 'u,
{
    fn borrow<'b>(&'b self) -> Ref<'b, T>
    where
        'u: 'b,
    {
        self.borrow_opt().unwrap()
    }
    fn borrow_mut<'b>(&'b self) -> RefMut<'b, T>
    where
        'u: 'b,
    {
        self.borrow_opt_mut().unwrap()
    }
    fn borrow_mut_custom<'b>(
        &'b self,
        wake_outside: bool,
        wake_here: bool,
        wake_inside: bool,
    ) -> RefMut<'b, T>
    where
        'u: 'b,
    {
        self.borrow_opt_mut_custom(wake_outside, wake_here, wake_inside)
            .unwrap()
    }
}

impl<'u, T, N> Tracked<'u, T> for N
where
    N: NodeDownTrait<'u, T>,
    T: 'u,
{
    fn borrow_opt<'b>(&'b self) -> Option<Ref<'b, T>>
    where
        'u: 'b,
    {
        self.node_up().up_borrow()
    }

    fn borrow_opt_mut_custom<'b>(
        &'b self,
        wake_outside: bool,
        wake_here: bool,
        wake_inside: bool,
    ) -> Option<RefMut<'b, T>>
    where
        'u: 'b,
    {
        let node = self.node_up();
        let b = node.up_borrow_mut();
        if b.is_some() {
            if wake_outside {
                node.invalidate_up();
            }
            if wake_here {
                node.get_listener().here().increment_version();
            }
            if wake_inside {
                self.invalidate_down();
            }
        }
        b
    }

    fn until_change_full<'a>(&'a self, inside: bool, here: bool, outside: bool) -> UntilChange<'a>
    where
        'u: 'a,
    {
        UntilChange::new(self.node_up().get_listener(), inside, here, outside)
    }
}
impl<'u, T, N> TrackedGuaranteed<'u, T> for N
where
    N: NodeDownTrait<'u, T> + IsGuaranteed<true>,
    T: 'u,
{
}

pub type Store<'store, Data, const GUARANTEED: bool> =
    <Data as crate::trackable::Trackable>::NodeDown<'store, GUARANTEED>;
