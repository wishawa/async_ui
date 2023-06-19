pub(crate) struct MiniScopeGuard<F: FnMut()>(pub F);
impl<F: FnMut()> Drop for MiniScopeGuard<F> {
    fn drop(&mut self) {
        (self.0)();
    }
}
