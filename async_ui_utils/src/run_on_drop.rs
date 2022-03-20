pub struct RunOnDrop<F: FnOnce()> {
    func: Option<F>,
}
impl<F: FnOnce()> RunOnDrop<F> {
    pub fn new(closure: F) -> Self {
        Self {
            func: Some(closure),
        }
    }
}
impl<F: FnOnce()> Drop for RunOnDrop<F> {
    fn drop(&mut self) {
        self.func.take().unwrap()();
    }
}
