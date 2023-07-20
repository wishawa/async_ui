pub mod bind_for_each;
pub mod for_each;
pub mod signal_stream;

use std::cell::{Ref, RefMut};

use futures_core::Stream;

use crate::{
    borrow_mut_guard::BorrowMutGuard, until_change::UntilChange, Path, ReferencePath, Trackable,
};

/// An extension trait with methods for [Path].
///
/// Most of the time, you'll be working with methods from this trait
/// (rather than the ones from [Path]).
///
/// This trait is implemented for every type that implements [Path].
pub trait PathExt: Path {
    /// Borrow the data at this path immutably.
    ///
    /// Returns None if the data identified by this path does not exist at
    /// the moment. For example...
    /// *   The path might point to data in an enum variant,
    ///     but the enum might be in a different variant.
    /// *   The path might point to data associated with a specific key in
    ///     a HashMap, but the HashMap might not have data at that key.
    ///
    /// If there is an existing mutable borrow anywhere in the
    /// store, this method will panic.
    ///
    /// See also: [borrow][crate::PathExtGuaranteed::borrow] - like this method,
    /// but for cases where we know None won't be returned.
    /// ```
    /// # use x_bow::{Store, PathExt, Trackable};
    /// #[derive(Trackable)]
    /// #[track(deep)]
    /// enum MyEnum {
    ///     Variant1(i32),
    ///     Variant2(String)
    /// }
    /// let store = Store::new(MyEnum::Variant1(5));
    ///
    /// let path_to_var1 = store.build_path().Variant1_0();
    /// // Borrow the `i32` inside the first enum variant.
    /// assert_eq!(path_to_var1.borrow_opt().as_deref(), Some(&5));
    ///
    /// let path_to_var2 = store.build_path().Variant2_0();
    /// // Can't borrow the `String`; the enum is in another variant.
    /// assert!(path_to_var2.borrow_opt().is_none());
    /// ```
    ///
    /// #### Time Complexity
    /// O(L) where L is the length of this path
    fn borrow_opt(&self) -> Option<Ref<'_, <Self as Path>::Out>> {
        self.path_borrow()
    }

    /// Borrow the data at this path mutably, notifying all the relevant
    /// change listeners when the returned borrow guard is dropped.
    ///
    /// Returns None if the data identified by this path does not exist at
    /// the moment. For example...
    /// *   The path might point to data in an enum variant,
    ///     but the enum might be in a different variant.
    /// *   The path might point to data associated with a specific key in
    ///     a HashMap, but the HashMap might not have data at that key.
    ///
    /// If there is an existing mutable or immutable borrow anywhere in the
    /// store, this method will panic.
    ///
    /// See also: [borrow_mut][crate::PathExtGuaranteed::borrow_mut] -
    /// like this method, but for cases where we know None won't be returned.
    ///
    /// ```
    /// # use x_bow::{Store, PathExt, Trackable};
    /// #[derive(Trackable)]
    /// #[track(deep)]
    /// enum MyEnum {
    ///     Variant1(i32),
    ///     Variant2(String)
    /// }
    /// let store = Store::new(MyEnum::Variant1(5));
    ///
    /// let path_to_var1 = store.build_path().Variant1_0();
    /// // The enum is in the first variant so we are sure we can borrow
    /// // and mutate the `i32`.
    /// *path_to_var1.borrow_opt_mut().unwrap() += 1;
    /// assert_eq!(path_to_var1.borrow_opt().as_deref(), Some(&6));
    ///
    /// let path_to_var2 = store.build_path().Variant2_0();
    /// assert!(path_to_var2.borrow_opt_mut().is_none());
    /// ```
    ///
    /// #### Time Complexity
    /// O(L + D_1 + D_2 + ... + D_N)
    /// where L is the length of this path
    /// and D_i is the length of the path of the ith subscriber
    /// (N is the number of subscribers that will be woken by the mutation,
    /// **not** the total number of subscribers in the store)
    fn borrow_opt_mut(&self) -> Option<BorrowMutGuard<'_, Self>> {
        self.path_borrow_mut()
            .map(|inner| BorrowMutGuard::new(inner, self.store_wakers(), self))
    }

    /// Borrow the data at this path mutably **without notifying** any listener.
    ///
    /// Use this in conjunction with the [notify_changed][PathExt::notify_changed] method.
    fn borrow_opt_mut_without_notifying(&self) -> Option<RefMut<'_, <Self as Path>::Out>> {
        self.path_borrow_mut()
    }

    /// Notify all listeners that the data at this location has changed.
    ///
    /// This method is not needed when using [borrow_opt_mut][PathExt::borrow_opt_mut] or
    /// [borrow_mut][crate::PathExtGuaranteed::borrow_mut]; in those cases
    /// notifications are sent automatically.
    ///
    /// This method is useful in the relatively rare situations when you need
    /// [borrow_opt_mut_without_notifying][PathExt::borrow_opt_mut_without_notifying].
    ///
    /// #### Time Complexity
    /// Same as [borrow_opt_mut][PathExt::borrow_opt_mut].
    fn notify_changed(&self) {
        crate::borrow_mut_guard::notify(self.store_wakers(), self);
    }

    /// Clone the data identified by this path.
    ///
    /// Equivalent to `path.borrow_opt().as_deref().map(Clone::clone)`.
    fn get_cloned(&self) -> Option<Self::Out>
    where
        Self::Out: Clone,
    {
        self.borrow_opt().as_deref().map(Clone::clone)
    }

    /// Set the data identified by this path, notifying listeners.
    ///
    /// Returns whether or not the data was set.
    ///
    /// Equivalent to `self.borrow_opt_mut().as_deref_mut().map(|s| *s = data).is_some()`.
    fn set(&self, data: Self::Out) -> bool
    where
        Self::Out: Sized,
    {
        self.borrow_opt_mut()
            .as_deref_mut()
            .map(|s| *s = data)
            .is_some()
    }

    /// Get a [Stream][futures_core::Stream] that fires everytime a mutable
    /// borrow is taken of this or any encompassing piece of data.
    ///
    /// In other words, whenever someone call [borrow_opt_mut][Self::borrow_opt_mut]
    /// or [borrow_mut][crate::PathExtGuaranteed::borrow_mut] on this path
    /// (the same one you're calling `until_change` on) or any path that is a
    /// prefix of this one, the stream will fire.
    ///
    /// ```
    /// # use x_bow::{Trackable, Store, PathExt};
    /// #[derive(Default, Trackable)]
    /// struct MyStruct {
    ///     field_1: i32,
    ///     field_2: u64
    /// }
    /// let store = Store::new(MyStruct::default());
    ///
    /// // path to the `MyStruct` itself
    /// let path = store.build_path();
    ///
    /// let stream = path.until_change();
    ///
    /// path.borrow_opt_mut(); // will fire the stream
    /// path.field_1().borrow_opt_mut(); // will fire the stream
    /// path.field_2().borrow_opt_mut(); // won't fire the stream
    /// ```
    ///
    /// ### Stream Behavior
    /// #### Spurious Fires
    /// The stream may fire spuriously, although the chance of this happening
    /// is [extremely low][crate#hash-collision].
    ///
    /// #### Batching
    /// If multiple changes happen in quick succession
    /// ("quick succession" means in-between calls to [Stream::poll_next], to
    /// be precise), the stream may only fire once.
    ///
    /// This means the stream can be used for detecting changes,
    /// but **not** for counting how many changes happened.
    ///
    /// #### Time Complexity
    /// On creation and each [poll][futures_core::stream::Stream::poll_next]:
    /// O(L) where L is the length of this path
    #[must_use = "the returned Stream is lazy; poll it or use StreamExt on it"]
    fn until_change(&self) -> UntilChange<'_> {
        UntilChange::new(self.store_wakers(), self)
    }

    /// Get a [Stream][futures_core::Stream] that fires **once now** and once
    /// every time the data changes, yielding [Ref]s to the data.
    ///
    /// The stream ends when the data cannot be borrowed (when [borrow_opt][Self::borrow_opt]
    /// returns None).
    ///
    /// This stream is useful for linking the data to some side-effect.
    /// For example, you can sync a UI widget content with the data...
    /// ```
    /// use futures_lite::StreamExt;
    /// # use x_bow::{Path, PathExt};
    /// # use std::cell::Ref;
    /// # async fn example(path: &impl Path<Out = String>) {
    /// # struct ui_element;
    /// # impl ui_element {
    /// #   fn set_text(&self, t: &str) {}
    /// # }
    /// // set the text with the current data
    /// // and update the text whenever the data change
    /// path.signal_stream().for_each(|data: Ref<'_, String>| {
    ///     let s: &str = &**data;
    ///     ui_element.set_text(s);
    /// }).await;
    /// # }
    /// ```
    /// **Note** that if this is all you need, use [for_each][PathExt::for_each]
    /// instead.
    ///
    /// ### Panic Opportunity
    /// This method returns a [Ref]. It is ‼️**extremely important**‼️ that you
    /// use the Ref and drop it as soon as possible. While the Ref is held,
    /// any attempt to mutably borrow any data in the store will **panic**.
    /// Holding the Ref for a long time (especially across a `await` points)
    /// is thus a very bad idea.
    ///
    /// ### Equivalent Behavior with Streams
    /// This method is merely for convenience. You can achieve the same
    /// functionality yourself like so...
    /// ```
    /// # use futures_lite::StreamExt;
    /// # use x_bow::{Path, PathExt};
    /// # use std::cell::Ref;
    /// # async fn example(path: &impl Path) {
    /// let stream = path.signal_stream();
    /// // is equivalent to
    /// let stream = futures_lite::stream::once(()) // fire once in the beginning...
    ///     .chain(path.until_change()) // and after every change
    ///     .map(|_| path.borrow_opt()) // borrow the data into a Ref
    ///     .take_while(Option::is_some) // end the stream if the data is unavailable
    ///     .map(Option::unwrap); // we've already checked that the data isn't none
    /// # }
    /// ```
    #[must_use = "the returned Stream is lazy; poll it or use StreamExt on it"]
    fn signal_stream(&self) -> signal_stream::SignalStream<'_, Self> {
        signal_stream::SignalStream::new(self, self.until_change())
    }

    /// Execute the given function with the data as argument. Repeat every time
    /// the data changes.
    ///
    /// The return future finishes when the data couldn't be accessed
    /// (when [borrow_opt][PathExt::borrow_opt] returns None).
    ///
    /// ```
    /// # use x_bow::{Path, PathExt};
    /// # async fn example(path: &impl Path<Out = String>) {
    /// # struct ui_element;
    /// # impl ui_element {
    /// #   fn set_text(&self, t: &str) {}
    /// # }
    /// // set the text with the current data
    /// // and update the text whenever the data change
    /// path.for_each(|s: &String| {
    ///     ui_element.set_text(s);
    /// }).await;
    /// # }
    /// ```
    ///
    /// This is a less powerful (and less panic-inducing) alternative to the
    /// [signal_stream][PathExt::signal_stream] method.
    #[must_use = "the returned Future is lazy; await it to make it do work"]
    fn for_each<C: FnMut(&Self::Out)>(&self, closure: C) -> for_each::ForEach<'_, Self, C> {
        for_each::ForEach::new(self, self.until_change(), closure)
    }

    /// For making a two-way binding.
    ///
    /// The given `on_change` function is called once in the beginning and
    /// whenever the data changes (just like with [for_each][PathExt::for_each]).
    ///
    /// Whenever the given `incoming_changes` [Stream] yield a value,
    /// the data will be updated to that value. The `on_change` function
    /// won't be called for changes that come in this way.
    ///
    /// The future finishes when either
    /// *   the `incoming_changes` stream yield None
    /// *   the data couldn't be accessed
    ///     ([borrow_opt][PathExt::borrow_opt] returns None)
    ///
    /// ```
    /// # use x_bow::{Path, PathExt};
    /// # use futures_lite::stream::{Stream, StreamExt, once};
    /// # async fn example(path: &impl Path<Out = String>) {
    /// # struct text_field_widget;
    /// # impl text_field_widget {
    /// #   fn set_text(&self, t: &str) {}
    /// #   fn until_change(&self) -> impl Stream<Item = ()> {once(())}
    /// #   fn get_text(&self) -> String {todo!()}
    /// # }
    /// path.bind_for_each(
    ///     // set the text with the current data
    ///     // and update the text whenever the data change
    ///     |s: &String| {
    ///         text_field_widget.set_text(s);
    ///     },
    ///     // whenever the user type into the field, update the data
    ///     text_field_widget
    ///         .until_change()
    ///         .map(|_| text_field_widget.get_text())
    /// ).await;
    /// # }
    /// ```
    #[must_use = "the returned Future is lazy; await it to make it do work"]
    fn bind_for_each<C: FnMut(&Self::Out), I: Stream<Item = Self::Out>>(
        &self,
        on_change: C,
        incoming_changes: I,
    ) -> bind_for_each::BindForEach<'_, Self, C, I> {
        bind_for_each::BindForEach::new(self, self.until_change(), on_change, incoming_changes)
    }

    /// Get a [Stream][futures_core::Stream] that fires everytime a mutable
    /// borrow is taken of this piece of data or anything inside it.
    ///
    /// In other words, whenever someone call [borrow_opt_mut][PathExt::borrow_opt_mut]
    /// or [borrow_mut][crate::PathExtGuaranteed::borrow_mut] on this path
    /// (the same one you're calling `until_bubbling_change` on) or any path that this
    /// path is a prefix of, the stream will fire.
    ///
    /// ```
    /// # use x_bow::{Trackable, Store, PathExt};
    /// #[derive(Trackable)]
    /// #[track(deep)]
    /// struct MyStruct<T> {
    ///     field_1: T,
    ///     field_2: u64
    /// }
    /// let store = Store::new(MyStruct {
    ///     field_1: MyStruct {
    ///         field_1: String::new(),
    ///         field_2: 123
    ///     },
    ///     field_2: 456
    /// });
    ///
    /// // path to the root `MyStruct` itself.
    /// let root = store.build_path();
    ///
    /// // path to the `field_1` in the root `MyStruct`.
    /// let listening = root.field_1();
    /// let stream = listening.until_bubbling_change();
    ///
    /// root.field_1().borrow_opt_mut(); // will fire the stream
    /// root.field_1().field_1().borrow_opt_mut(); // will fire the stream
    /// root.field_1().field_2().borrow_opt_mut(); // will fire the stream
    /// root.borrow_opt_mut(); // won't fire the stream
    /// root.field_2().borrow_opt_mut(); // won't fire the stream
    /// ```
    ///
    /// ### Stream Behavior
    /// The returned stream may spuriously fire or batch changes.
    /// See [until_change][PathExt::until_change] documentation.
    ///
    /// #### Time Complexity
    /// On creation:
    /// O(L) where L is the length of this path
    ///
    /// On each poll:
    /// O(1)
    #[must_use = "the returned Stream is lazy; poll it or use StreamExt on it"]
    fn until_bubbling_change(&self) -> UntilChange<'_> {
        UntilChange::new_bubbling(self.store_wakers(), self)
    }

    /// Gives a [PathBuilder][crate::Trackable::PathBuilder] for creating a
    /// path that continues from this path.
    ///
    /// This is useful when you are handling the type that implements `Path`
    /// directly. Most of the time, though, you will already be working with
    /// `PathBuilder`s.
    fn build_path(self) -> <Self::Out as Trackable>::PathBuilder<Self>
    where
        Self: Sized,
        Self::Out: Trackable,
    {
        <Self::Out as Trackable>::new_path_builder(self)
    }

    /// Create a new Path from borrowing this path.
    ///
    /// Usually, paths are owned and live for as long as the store itself.
    /// ```
    /// # use x_bow::{Trackable, Store, PathExt};
    /// # #[derive(Default, Trackable)]
    /// # struct MyStruct {
    /// #     field_1: i32,
    /// #     field_2: u64
    /// # }
    /// let store = Store::new(MyStruct::default());
    ///
    /// // `path_a` lives for as long as the store.
    /// let path_a = store.build_path();
    ///
    /// // `path_b` composes `path_a`.
    /// // `path_b` lives for as long as the store.
    /// let path_b = path_a.field_1();
    ///
    /// // `path_c` borrows `path_a`.
    /// // if you drop `path_a`, `path_c` will no longer be usable.
    /// let path_c = path_a.as_ref_path().field_1();
    /// ```
    ///
    /// Note that the new path will only live for as long as the borrow.
    /// This means it won't be useful for use cases requiring long-lived path
    /// (such as implementing an undo-redo log).
    fn as_ref_path(&self) -> <Self::Out as Trackable>::PathBuilder<ReferencePath<'_, Self>>
    where
        Self::Out: Trackable,
    {
        <Self::Out as Trackable>::new_path_builder(ReferencePath::new(self))
    }
}

/// Extension trait is blanket-implemented.
impl<P: Path + ?Sized> PathExt for P {}
