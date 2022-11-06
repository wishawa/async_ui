use std::cell::{Cell, UnsafeCell};
use std::future::Future;
use std::marker::PhantomPinned;
use std::mem::{ManuallyDrop, MaybeUninit};
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};

use pin_utils::pin_mut;
use scoped_async_spawn::{GiveUnforgettableScope, SpawnGuard};

fn poll_once<T>(f: impl Future<Output = T>) -> Option<T> {
    struct NullWaker;
    impl Wake for NullWaker {
        fn wake(self: Arc<Self>) {}
    }

    let waker = Waker::from(Arc::new(NullWaker));
    let mut context = Context::from_waker(&waker);
    pin_mut!(f);
    match f.poll(&mut context) {
        Poll::Ready(v) => Some(v),
        Poll::Pending => None,
    }
}

fn with_context<R>(f: impl Unpin + FnMut(&mut Context<'_>) -> Poll<R>) -> impl Future<Output = R> {
    struct WithContext<F>(F);
    impl<F, R> Future for WithContext<F>
    where
        F: FnMut(&mut Context<'_>) -> Poll<R> + Unpin,
    {
        type Output = R;

        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            (self.0)(cx)
        }
    }
    WithContext(f)
}

#[derive(Debug)]
struct IsDropped {
    was_dropped: bool,
}
impl IsDropped {
    fn new() -> Self {
        Self { was_dropped: false }
    }
}
impl Drop for IsDropped {
    fn drop(&mut self) {
        self.was_dropped = true;
    }
}

/// This will panic with "Not in scope." despite the fact that the guard is
/// inside a scope.
///
/// The reason for this is because there is no yield/await points inside the
/// future so the `guard` is stored on the thread's stack instead of inside the
/// future's memory. Therefore the `Pointer::contains` method will return
/// `false` since the thread stack is not stored inside the
/// `GiveUnforgettableScope` future.
#[test]
#[should_panic]
fn incorrect_not_in_scope() {
    poll_once(GiveUnforgettableScope::new_static(async move {
        let guard = SpawnGuard::new();
        pin_mut!(guard);
        let _ = guard.convert_future(async {});
    }));
}

/// This crate relies on the assumption that stack allocated values can't be
/// forgotten. This is not documented as a guarantee for the `Pin` type but it
/// seems to be a property that is upheld by most crates.
///
/// This test demonstrates how this crate is unsound if forgettable stack
/// allocations are allowed.
///
/// # Forgettable stack allocation
///
/// Pin a value to the stack and then forget it. We don't free/reuse/invalidate
/// the memory of the value though so this should be allowed! It is the same
/// thing that `Box::pin` and `Box::leak` allows after all.
#[test]
fn forgettable_stack_allocations() {
    struct StackAlloc<const N: usize> {
        data: UnsafeCell<[MaybeUninit<u8>; N]>,
        used_length: Cell<usize>,
        pinned_count: Cell<usize>,
        _phantom_pin: PhantomPinned,
    }
    impl<const N: usize> StackAlloc<N> {
        #[inline(always)]
        pub const fn new() -> Self {
            Self {
                data: UnsafeCell::new([MaybeUninit::uninit(); N]),
                used_length: Cell::new(0),
                pinned_count: Cell::new(0),
                _phantom_pin: PhantomPinned,
            }
        }
        fn ptr(&self) -> *mut u8 {
            self.data.get().cast()
        }
        pub fn alloc<T>(&self, value: T) -> StackBox<'_, T> {
            let used_len = self.used_length.get();
            let first_free = unsafe { self.ptr().add(used_len) };

            let align_fix = first_free.align_offset(std::mem::align_of::<T>());
            let new_used_len = used_len
                .saturating_add(align_fix)
                .saturating_add(std::mem::size_of::<T>());
            if new_used_len > N || new_used_len >= isize::MAX as usize {
                panic!("StackAlloc: out of memory");
            }

            let value_ptr = unsafe { first_free.add(align_fix) }.cast::<T>();
            unsafe { value_ptr.write(value) };
            self.used_length.set(new_used_len);
            StackBox {
                data: unsafe { &mut *value_ptr.cast::<ManuallyDrop<T>>() },
                pinned_count: None,
            }
        }
        pub fn alloc_pinned<T>(self: Pin<&Self>, value: T) -> Pin<StackBox<'_, T>> {
            let mut value = self.get_ref().alloc(value);

            // Track pinned allocations, so that we don't free the backing memory if
            // they are leaked:
            self.pinned_count
                .set(self.pinned_count.get().checked_add(1).unwrap());
            value.pinned_count = Some(&self.get_ref().pinned_count);

            // Safety: we don't move the allocation and are careful to uphold the
            // drop guarantees.
            unsafe { Pin::new_unchecked(value) }
        }
    }
    impl<const N: usize> Drop for StackAlloc<N> {
        fn drop(&mut self) {
            if self.pinned_count.get() != 0 {
                struct AbortBomb;
                impl Drop for AbortBomb {
                    fn drop(&mut self) {
                        std::process::abort();
                    }
                }

                let _bomb = AbortBomb;
                panic!(
                    "Aborting process because {} pinned stack allocation(s) were leaked.",
                    self.pinned_count.get()
                );
            }
        }
    }

    struct StackBox<'a, T> {
        data: &'a mut ManuallyDrop<T>,
        pinned_count: Option<&'a Cell<usize>>,
    }
    impl<T> Deref for StackBox<'_, T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            self.data
        }
    }
    impl<T> DerefMut for StackBox<'_, T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            self.data
        }
    }
    impl<T> Drop for StackBox<'_, T> {
        fn drop(&mut self) {
            unsafe { ManuallyDrop::drop(self.data) };
            if let Some(pinned_count) = self.pinned_count {
                pinned_count.set(pinned_count.get() - 1);
            }
        }
    }

    let scoped = GiveUnforgettableScope::new_static(async move {
        let buffer = StackAlloc::<1024>::new();
        pin_mut!(buffer);

        let mut static_fut = None;
        {
            let data = String::from("some data that might be freed");
            let data2 = IsDropped::new();

            let mut inner = buffer.as_ref().alloc_pinned(async {
                {
                    let guard = SpawnGuard::new();
                    pin_mut!(guard);
                    static_fut = Some(guard.convert_future(async {
                        println!("{data}");
                        println!("{data2:?}");
                        assert!(
                            data2.was_dropped,
                            "observing data after it was dropped, not good!"
                        );
                    }));
                    with_context::<()>(|_| Poll::Pending).await
                }
            });
            poll_once(inner.as_mut());
            std::mem::forget(inner);
        }
        static_fut
            .expect("converted a future to static lifetime")
            .await;
        // Yield forever so that `buffer` isn't dropped which would abort the process:
        with_context::<()>(|_| Poll::Pending).await;
    });
    let mut scoped = Box::pin(scoped);
    poll_once(scoped.as_mut());
    // Leak the future so that `buffer` isn't dropped which would abort the
    // process
    std::mem::forget(scoped);
}
