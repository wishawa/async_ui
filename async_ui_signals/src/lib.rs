#![feature(generic_associated_types)]
#![forbid(unsafe_op_in_unsafe_fn)]

use sub::{SubManager, ParentSub};

mod sub;
pub mod nodes;

mod push_based;

pub trait Visitable<V>
where
    V: ?Sized,
{
    fn visit<'v, 's>(&'s self, visitor: &'v mut V)
    where
        Self: 'v
    ;

    fn get_sub<'s>(&'s self) -> ParentSub<'s>;
}
