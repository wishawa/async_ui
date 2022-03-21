use crate::elem::Elem;

impl<'a, H: Clone + 'a> Elem<'a, H> {
    pub fn reference<'x>(&'x self) -> H {
        let h: &H = &self.elem;
        h.clone()
    }
}
