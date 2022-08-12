use std::{
    cell::RefMut,
    ops::{Deref, DerefMut},
};

use crate::{
    __private_macro_only::TrackedEdge,
    tracked::{Tracked, TrackedNode},
};
pub struct NotifyGuard<'b, N>
where
    N: TrackedNode,
{
    pub(crate) inside: RefMut<'b, <N::Edge as TrackedEdge>::Data>,
    pub(crate) tracked: &'b Tracked<N>,
}

impl<'b, N> Deref for NotifyGuard<'b, N>
where
    N: TrackedNode,
{
    type Target = <N::Edge as TrackedEdge>::Data;

    fn deref(&self) -> &Self::Target {
        self.inside.deref()
    }
}

impl<'b, N> DerefMut for NotifyGuard<'b, N>
where
    N: TrackedNode,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inside.deref_mut()
    }
}

impl<'b, N> Drop for NotifyGuard<'b, N>
where
    N: TrackedNode,
{
    fn drop(&mut self) {
        self.tracked.invalidate_inside_up();
        self.tracked.invalidate_outside_down();
    }
}
