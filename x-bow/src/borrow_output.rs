use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use crate::{
    deref_optional::{ProjectedDeref, ProjectedDerefMut},
    tracked::TrackedNode,
};

pub trait MutabilityFlag {
    fn on_mutated(&mut self);
}
pub struct Mutable<'p, P: TrackedNode + ?Sized>(pub(crate) &'p P);

impl<'p, P: TrackedNode + ?Sized> MutabilityFlag for Mutable<'p, P> {
    fn on_mutated(&mut self) {
        self.0.invalidate_inside_up();
        self.0.invalidate_outside_down();
    }
}

pub struct NotMutable(pub(crate) PhantomData<()>);
impl MutabilityFlag for NotMutable {
    fn on_mutated(&mut self) {}
}

pub struct XBowBorrow<M, G>
where
    G: ProjectedDeref,
    M: MutabilityFlag,
{
    guard: G,
    mutable: M,
}

impl<M, G> XBowBorrow<M, G>
where
    G: ProjectedDeref,
    M: MutabilityFlag,
{
    pub(crate) fn new(guard: G, mutable: M) -> Option<Self> {
        if guard.deref_optional().is_some() {
            Some(Self::new_without_check(guard, mutable))
        } else {
            None
        }
    }
    pub(crate) fn new_without_check(guard: G, mutable: M) -> Self {
        Self { guard, mutable }
    }
}

impl<M, G> Drop for XBowBorrow<M, G>
where
    G: ProjectedDeref,
    M: MutabilityFlag,
{
    fn drop(&mut self) {
        self.mutable.on_mutated();
    }
}

impl<M, G> Deref for XBowBorrow<M, G>
where
    G: ProjectedDeref,
    M: MutabilityFlag,
{
    type Target = G::Target;
    fn deref(&self) -> &Self::Target {
        self.guard.deref_optional().unwrap()
    }
}

impl<'p, P, G> DerefMut for XBowBorrow<Mutable<'p, P>, G>
where
    G: ProjectedDerefMut,
    P: TrackedNode + ?Sized,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.guard.deref_mut_optional().unwrap()
    }
}
// impl<'p, G, P> XBowBorrow<'p, G, P>
// where
//     G: ProjectedDeref,
//     P: TrackedNode + ?Sized,
// {
//     pub(crate) fn new(guard: G, projection: Option<&'p P>) -> Option<Self> {
//         if guard.deref_optional().is_some() {
//             Some(Self::new_without_check(guard, projection))
//         } else {
//             None
//         }
//     }
//     pub(crate) fn new_without_check(guard: G, projection: Option<&'p P>) -> Self {
//         Self { guard, projection }
//     }
// }

// impl<'p, G, P> Deref for XBowBorrow<'p, G, P>
// where
//     G: ProjectedDeref,
//     P: TrackedNode + ?Sized,
// {
//     type Target = <G as ProjectedDeref>::Target;

//     fn deref(&self) -> &Self::Target {
//         self.guard.deref_optional().unwrap()
//     }
// }
// impl<'p, G, P> DerefMut for XBowBorrow<'p, G, P>
// where
//     G: ProjectedDerefMut,
//     P: TrackedNode + ?Sized,
// {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         self.guard.deref_mut_optional().unwrap()
//     }
// }
// impl<'p, G, P> Drop for XBowBorrow<'p, G, P>
// where
//     G: ProjectedDeref,
//     P: TrackedNode + ?Sized,
// {
//     fn drop(&mut self) {
//         if let Some(proj) = self.projection {
//             proj.invalidate_inside_up();
//             proj.invalidate_outside_down();
//         }
//     }
// }
