use core::task::{RawWaker, RawWakerVTable, Waker};

/// A Waker that doesn't do anything.
pub fn dummy_waker() -> Waker {
    fn new_raw_waker() -> RawWaker {
        unsafe fn no_op(_data: *const ()) {}
        unsafe fn clone(_data: *const ()) -> RawWaker {
            new_raw_waker()
        }
        static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, no_op, no_op, no_op);
        RawWaker::new(core::ptr::null(), &VTABLE)
    }
    unsafe { Waker::from_raw(new_raw_waker()) }
}
