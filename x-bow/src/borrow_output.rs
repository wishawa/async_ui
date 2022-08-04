use std::ops::{Deref, DerefMut};

use crate::{
    deref_optional::{ProjectedDeref, ProjectedDerefMut},
    tracked::Tracked,
};

pub struct XBowBorrow<'p, G, P>
where
    G: ProjectedDeref,
    P: Tracked + ?Sized,
{
    guard: G,
    projection: Option<&'p P>,
}

impl<'p, G, P> XBowBorrow<'p, G, P>
where
    G: ProjectedDeref,
    P: Tracked + ?Sized,
{
    pub(crate) fn new(guard: G, projection: Option<&'p P>) -> Option<Self> {
        if guard.deref_optional().is_some() {
            Some(Self::new_without_check(guard, projection))
        } else {
            None
        }
    }
    pub(crate) fn new_without_check(guard: G, projection: Option<&'p P>) -> Self {
        Self { guard, projection }
    }
}

impl<'p, G, P> Deref for XBowBorrow<'p, G, P>
where
    G: ProjectedDeref,
    P: Tracked + ?Sized,
{
    type Target = <G as ProjectedDeref>::Target;

    fn deref(&self) -> &Self::Target {
        self.guard.deref_optional().unwrap()
    }
}
impl<'p, G, P> DerefMut for XBowBorrow<'p, G, P>
where
    G: ProjectedDerefMut,
    P: Tracked + ?Sized,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.guard.deref_mut_optional().unwrap()
    }
}
impl<'p, G, P> Drop for XBowBorrow<'p, G, P>
where
    G: ProjectedDeref,
    P: Tracked + ?Sized,
{
    fn drop(&mut self) {
        if let Some(proj) = self.projection {
            proj.invalidate_up_inside();
            proj.invalidate_down_outside();
        }
    }
}
