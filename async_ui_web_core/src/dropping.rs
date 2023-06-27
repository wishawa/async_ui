//! For handling unmounting HTML elements.
//!
//! When a [ContainerNodeFuture][crate::ContainerNodeFuture] is dropped,
//! it removes its HTML node from the parent HTML node.
//! If we let things happen naturally, the drop glue would then call the drop
//! impl of the descendant futures, causing them to detach their HTML nodes.
//! However, the HTML nodes in these descendants are no longer in the HTML document
//! (because their ancestor has been removed), so we want to just ignore them.
//! The mechanism in this module let futures know whether or not their ancestor
//! have been removed.

use std::cell::Cell;

// We have a thread local that takes a non-null value when
// some dropping is going on and the ancestor has already been removed.
thread_local! (
    static IS_DROPPING: Cell<*const DetachmentBlocker> = Cell::new(std::ptr::null())
);

/// Use for accessing the thread local that determines whether or not DOM
/// futures being dropped should try to detach their nodes from the parent.
/// Rust's drop glue drops struct fields in order of declaration,
/// so as the last field, we put a struct that will unset the thread local
/// when dropped.
pub struct DetachmentBlocker;

impl DetachmentBlocker {
    /// Block the detachment, returning true iff it was already blocked.
    /// If the detachment was not previously blocked, then this instance of
    /// `DetachmentBlocker` will "own" the blocking, and detachment will be
    /// unblocked when this instance is dropped.
    pub fn block_until_drop(&self) -> bool {
        IS_DROPPING.with(|cell| {
            if cell.get().is_null() {
                cell.set(self as *const DetachmentBlocker);
                false
            } else {
                true
            }
        })
    }
}

impl Drop for DetachmentBlocker {
    fn drop(&mut self) {
        let addr = self as *const DetachmentBlocker;
        IS_DROPPING.with(|cell| {
            // Make sure we're the one meant to unset this thread local.
            if std::ptr::eq(cell.get(), addr) {
                cell.set(std::ptr::null());
            }
        });
    }
}
