use std::{
    cell::{RefCell, RefMut},
    rc::{Rc, Weak},
};

pub mod backend;
pub mod control;
pub mod drop_check;
pub mod element;
pub mod render;
pub mod wrappers;

pub trait MaybeSend {}
impl<T> MaybeSend for T {}

pub trait MaybeSync {}
impl<T> MaybeSync for T {}

type Shared<T> = Rc<T>;
type SharedWeak<T> = Weak<T>;
type Mutable<T> = RefCell<T>;
fn mutable_borrow_mut<T>(m: &Mutable<T>) -> RefMut<'_, T> {
    m.borrow_mut()
}
