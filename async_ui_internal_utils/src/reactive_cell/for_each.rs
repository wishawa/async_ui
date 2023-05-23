use super::ReactiveCell;

impl<T> ReactiveCell<T> {
    pub async fn for_each<F: FnMut(&T)>(&self, mut func: F) {
        loop {
            {
                func(&self.borrow());
            }
            self.until_change().await;
        }
    }
}
