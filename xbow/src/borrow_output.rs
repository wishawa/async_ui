use std::ops::{Deref, DerefMut};

use crate::deref_optional::{ProjectedDeref, ProjectedDerefMut};

pub struct XBowBorrow<G>
where
    G: ProjectedDeref,
{
    guard: G,
}

impl<G> XBowBorrow<G>
where
    G: ProjectedDeref,
{
    pub(crate) fn new(guard: G) -> Option<Self> {
        if guard.deref_optional().is_some() {
            Some(Self { guard })
        } else {
            None
        }
    }
}

impl<G> Deref for XBowBorrow<G>
where
    G: ProjectedDeref,
{
    type Target = <G as ProjectedDeref>::Target;

    fn deref(&self) -> &Self::Target {
        self.guard.deref_optional().unwrap()
    }
}
impl<G> DerefMut for XBowBorrow<G>
where
    G: ProjectedDerefMut,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.guard.deref_mut_optional().unwrap()
    }
}
impl<G> Drop for XBowBorrow<G>
where
    G: ProjectedDeref,
{
    fn drop(&mut self) {
        self.guard.fire_listeners();
    }
}
