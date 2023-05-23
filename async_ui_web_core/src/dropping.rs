//! For handling unmounting HTML elements.
//!
//! When a [ContainerNodeFuture][crate::ContainerNodeFuture] is dropped,
//! it removes its HTML element from its parent HTML element.
//! But then the drop glue drops its descendant futures.
//! The nodes in these descendants are no longer in the HTML document
//! (because their ancestor has been removed), so we want to just ignore them.

use std::cell::Cell;

// To acheive the above, we have a thread local that takes a non-null value when
// some dropping is going on and the ancestor has already been removed.
thread_local! (
    static IS_DROPPING: Cell<*const UnsetIsDropping> = Cell::new(std::ptr::null())
);

/// How do we set the thread local back to null when dropping is done?
/// Rust's drop glue drops struct fields in order of declaration,
/// so as the last field, we put a struct that will unset the thread local
/// when dropped.
pub(crate) struct UnsetIsDropping;

impl UnsetIsDropping {
    /// Set the thread local to a non-null value if not already set.
    /// The value is the address of the `UnsetIsDropping` object that will
    /// be the one to unset this.
    /// Returns false if the thread local was not already set.
    pub fn set_here(&self) -> bool {
        IS_DROPPING.with(|cell| {
            if cell.get().is_null() {
                cell.set(self as *const UnsetIsDropping);
                false
            } else {
                true
            }
        })
    }
}

impl Drop for UnsetIsDropping {
    fn drop(&mut self) {
        let addr = self as *const UnsetIsDropping;
        IS_DROPPING.with(|cell| {
            // Make sure we're the one meant to unset this thread local.
            if std::ptr::eq(cell.get(), addr) {
                cell.set(std::ptr::null());
            }
        });
    }
}
